//! The Xbox app's installed PC games, on Windows. The Xbox app has no
//! sign-in a program like UpNext may use, so its owned library (Game Pass
//! included) can't be read; what is installed can. The Xbox app installs a
//! game into a folder of its own under `XboxGames` at a drive's root (the
//! folder the hidden `.GamingRoot` file there names), and each game's files
//! hold a `MicrosoftGame.config`, which ordinary Store apps don't have, and
//! the package's `appxmanifest.xml`. The game starts through Windows from
//! its package family name and application id, as the Start menu starts it:
//! running its `.exe` directly fails for a Store package.
//!
//! Every file here is read as untrusted text: size-capped, its ids checked
//! against what Windows allows, and its executables never run.

use crate::db::ImportedGame;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// GOG Galaxy's name for the Xbox store, which games added by hand use too.
pub const SOURCE: &str = "xboxone";
/// The folder the Xbox app installs into when its drive's `.GamingRoot`
/// names none UpNext can read.
#[cfg_attr(not(windows), allow(dead_code))]
const GAMES_FOLDER: &str = "XboxGames";
/// The hidden file at the root of each drive the Xbox app installs to.
#[cfg_attr(not(windows), allow(dead_code))]
const MARKER: &str = ".GamingRoot";
/// The most of any one file read; real ones are a few kilobytes.
const MAX_FILE: u64 = 1 << 20;

/// One game the Xbox app installed.
#[derive(Debug, Clone, PartialEq)]
pub struct XboxInstall {
    /// The package family name (`Name_publisherid`), which Windows starts it by.
    pub family: String,
    /// The package's application id, the part after `!` in its launch link.
    pub app_id: Option<String>,
    /// The Microsoft Store's product id (`9NBLGGH4R315`), when the game has one.
    pub store_id: Option<String>,
    pub name: String,
}

/// The characters Windows writes a publisher id in: base 32, without `i`,
/// `l`, `o` and `u`.
const PUBLISHER_ID_DIGITS: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// The 13-character publisher id Windows puts after a package's name in its
/// family name, worked out from the publisher's distinguished name: the
/// first 8 bytes of the SHA-256 of its UTF-16 text, 5 bits at a time.
pub fn publisher_id(publisher: &str) -> String {
    let utf16: Vec<u8> = publisher.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let hash = Sha256::digest(&utf16);
    // 64 bits and one 0 bit make 13 digits of 5 bits.
    let bits = u128::from(u64::from_be_bytes(hash[..8].try_into().expect("8 bytes"))) << 1;
    (0..13).rev().map(|i| PUBLISHER_ID_DIGITS[((bits >> (i * 5)) & 31) as usize] as char).collect()
}

static PACKAGE_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9.\-]{3,50}$").expect("valid regex"));
static APP_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9.]{0,63}$").expect("valid regex"));
static STORE_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Za-z]{12}$").expect("valid regex"));
static RUN_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^shell:AppsFolder\\([A-Za-z0-9.\-]{3,50}_[0-9a-hjkmnp-tv-z]{13})!([A-Za-z][A-Za-z0-9.]{0,63})$").expect("valid regex")
});

/// The package family name for a package `name` from `publisher`, or `None`
/// when the name isn't one Windows allows.
pub fn family_name(name: &str, publisher: &str) -> Option<String> {
    let (name, publisher) = (name.trim(), publisher.trim());
    (PACKAGE_NAME.is_match(name) && !publisher.is_empty()).then(|| format!("{name}_{}", publisher_id(publisher)))
}

/// The link that starts the game through Windows.
pub fn run_url(family: &str, app_id: &str) -> String {
    format!(r"shell:AppsFolder\{family}!{app_id}")
}

/// `raw` as a launch link UpNext made, or `None` for anything else. Windows
/// opens any folder or program a `shell:` link names, so the desktop app
/// hands out only these, and only in exactly this form. A webview may have
/// percent-encoded the backslash or the `!` on the way.
#[cfg_attr(not(feature = "desktop"), allow(dead_code))]
pub fn run_link(raw: &str) -> Option<String> {
    let link = raw.replace("%5C", "\\").replace("%5c", "\\").replace("%21", "!");
    RUN_LINK.is_match(&link).then_some(link)
}

/// The game's page in the Microsoft Store, on the web.
pub fn store_url(store_id: &str) -> String {
    format!("https://apps.microsoft.com/detail/{store_id}")
}

static COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<!--.*?-->").expect("valid regex"));

/// The attributes of each `<tag …>` element in `xml`, in order. Comments are
/// dropped first, so a commented-out element doesn't count.
fn elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let re = Regex::new(&format!(r"<{tag}\b([^>]*)>")).expect("valid regex");
    re.captures_iter(xml).filter_map(|c| c.get(1)).map(|m| m.as_str()).collect()
}

/// An attribute's value in an element's attribute text.
fn attr(attrs: &str, name: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"(?:^|\s){name}\s*=\s*(?:"([^"]*)"|'([^']*)')"#)).expect("valid regex");
    let c = re.captures(attrs)?;
    let v = c.get(1).or_else(|| c.get(2))?.as_str();
    Some(unescape(v.trim())).filter(|v| !v.is_empty())
}

/// The text of the first `<tag>…</tag>` in `xml`.
fn text(xml: &str, tag: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?s)<{tag}>([^<]*)</{tag}>")).expect("valid regex");
    let v = re.captures(xml)?.get(1)?.as_str();
    Some(unescape(v.trim())).filter(|v| !v.is_empty())
}

/// XML's escapes, for names such as `Ori &amp; the Will of the Wisps`.
fn unescape(s: &str) -> String {
    static NUMERIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&#(x[0-9A-Fa-f]+|[0-9]+);").expect("valid regex"));
    let s = NUMERIC.replace_all(s, |c: &regex::Captures| {
        let n = &c[1];
        let code = match n.strip_prefix('x') {
            Some(hex) => u32::from_str_radix(hex, 16).ok(),
            None => n.parse().ok(),
        };
        code.and_then(char::from_u32).map(String::from).unwrap_or_default()
    });
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// A display name a person can read: the packages that localise theirs give
/// an `ms-resource:` reference instead, which only Windows can look up.
fn readable(name: Option<String>) -> Option<String> {
    name.filter(|n| !n.to_lowercase().starts_with("ms-resource:"))
}

/// A file's text, or `None` when it's missing, too large or not text.
fn read_capped(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_FILE {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    // Windows' tools write these files in UTF-8, sometimes with a byte order mark.
    let text = String::from_utf8(bytes).ok()?;
    Some(text.trim_start_matches('\u{feff}').to_string())
}

/// The game in one folder the Xbox app installed into (`XboxGames\Halo`),
/// or `None` when it holds no game. Its files are in the folder's `Content`
/// folder, or in the folder itself.
pub fn read_game(folder: &Path) -> Option<XboxInstall> {
    let files = [folder.join("Content"), folder.to_path_buf()].into_iter().find(|d| d.join("MicrosoftGame.config").is_file())?;
    let config = COMMENT.replace_all(&read_capped(&files.join("MicrosoftGame.config"))?, "").into_owned();
    let manifest = read_capped(&files.join("appxmanifest.xml")).map(|m| COMMENT.replace_all(&m, "").into_owned());

    // Windows registered the package under the manifest's identity; the
    // game's config holds the same one, for a manifest that can't be read.
    let identity = |xml: &str| {
        let attrs = *elements(xml, "Identity").first()?;
        family_name(&attr(attrs, "Name")?, &attr(attrs, "Publisher")?)
    };
    let family = manifest.as_deref().and_then(identity).or_else(|| identity(&config))?;
    let app_id = manifest
        .as_deref()
        .and_then(|m| elements(m, "Application").first().and_then(|a| attr(a, "Id")))
        .or_else(|| {
            // The config's executables for PC, the first of which the
            // package's application is made from.
            elements(&config, "Executable")
                .into_iter()
                .filter(|e| attr(e, "TargetDeviceFamily").is_none_or(|f| f.eq_ignore_ascii_case("PC")))
                .find_map(|e| attr(e, "Id"))
        })
        .filter(|id| APP_ID.is_match(id));
    let store_id = text(&config, "StoreId").filter(|id| STORE_ID.is_match(id)).map(|id| id.to_uppercase());
    let shown = elements(&config, "ShellVisuals").first().and_then(|v| attr(v, "DefaultDisplayName"));
    let name = readable(shown)
        .or_else(|| readable(manifest.as_deref().and_then(|m| text(m, "DisplayName"))))
        .or_else(|| folder.file_name().and_then(|f| f.to_str()).map(String::from))?;
    Some(XboxInstall { family, app_id, store_id, name })
}

/// The games in the Xbox app's install folders `roots`, each once.
pub fn installed(roots: &[PathBuf]) -> Vec<XboxInstall> {
    let mut out: Vec<XboxInstall> = vec![];
    for root in roots {
        let Ok(list) = std::fs::read_dir(root) else { continue };
        let mut folders: Vec<PathBuf> = list.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect();
        folders.sort();
        for g in folders.iter().filter_map(|f| read_game(f)) {
            if !out.iter().any(|o| o.family.eq_ignore_ascii_case(&g.family)) {
                out.push(g);
            }
        }
    }
    out
}

/// The install folders a drive's `.GamingRoot` names, relative to the
/// drive's root. The file is a short header (`RGBX` and a count) and then
/// UTF-16 folder names, each ended by a 0. A name that would leave the
/// drive's root is dropped.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn gaming_root_folders(bytes: &[u8]) -> Vec<String> {
    let body = if bytes.starts_with(b"RGBX") { bytes.get(8..).unwrap_or_default() } else { bytes };
    let units: Vec<u16> = body.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
    units
        .split(|u| *u == 0)
        .filter_map(|name| String::from_utf16(name).ok())
        .map(|name| name.trim().trim_matches(['\\', '/']).to_string())
        .filter(|name| {
            !name.is_empty()
                && !name.contains(':')
                && !name.chars().any(char::is_control)
                && !name.split(['\\', '/']).any(|part| part.is_empty() || part == "." || part == "..")
        })
        .collect()
}

/// The Xbox app's install folders on this computer: each drive's
/// `.GamingRoot` folders, or its `XboxGames` when it names none. Tests never
/// read this computer's drives (as with the registry).
pub fn roots() -> Vec<PathBuf> {
    if cfg!(test) { vec![] } else { read_roots() }
}

#[cfg(windows)]
fn read_roots() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = vec![];
    // A and B were floppy drives, which can ask for a disk.
    for letter in 'C'..='Z' {
        let drive = PathBuf::from(format!(r"{letter}:\"));
        let marker = drive.join(MARKER);
        let named = std::fs::metadata(&marker)
            .ok()
            .filter(|m| m.is_file() && m.len() <= MAX_FILE)
            .and_then(|_| std::fs::read(&marker).ok())
            .map(|b| gaming_root_folders(&b))
            .unwrap_or_default();
        let folders = if named.is_empty() { vec![GAMES_FOLDER.to_string()] } else { named };
        for f in folders {
            let dir = drive.join(f);
            if dir.is_dir() && !out.contains(&dir) {
                out.push(dir);
            }
        }
    }
    out
}

#[cfg(not(windows))]
fn read_roots() -> Vec<PathBuf> {
    vec![]
}

pub fn copy(g: &XboxInstall) -> ImportedGame {
    ImportedGame {
        source: SOURCE.into(),
        // The Store's id when there is one, as GOG Galaxy's Xbox copies have.
        source_id: g.store_id.as_deref().map(str::to_lowercase).unwrap_or_else(|| g.family.clone()),
        launcher: SOURCE.into(),
        name: g.name.clone(),
        installed: true,
        store_url: g.store_id.as_deref().map(store_url),
        run_url: g.app_id.as_deref().map(|app| run_url(&g.family, app)),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MICROSOFT: &str = "CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US";

    fn config(identity: &str, extra: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<Game configVersion="1">
  <!-- <Identity Name="Commented.Out" Publisher="CN=Nobody" /> -->
  {identity}
  <ExecutableList>
    <Executable Name="GameXbox.exe" Id="XboxGame" TargetDeviceFamily="Scarlett" />
    <Executable Name="Game.exe" Id="Game" TargetDeviceFamily="PC" />
  </ExecutableList>
  {extra}
</Game>"#
        )
    }

    fn game(root: &Path, folder: &str, config: &str, manifest: Option<&str>) {
        let content = root.join(folder).join("Content");
        std::fs::create_dir_all(&content).unwrap();
        std::fs::write(content.join("MicrosoftGame.config"), config).unwrap();
        if let Some(m) = manifest {
            std::fs::write(content.join("appxmanifest.xml"), m).unwrap();
        }
    }

    #[test]
    fn publisher_ids_match_windows() {
        // Every package Microsoft publishes ends in this one.
        assert_eq!(publisher_id(MICROSOFT), "8wekyb3d8bbwe");
        assert_eq!(family_name("Microsoft.WindowsCalculator", MICROSOFT).as_deref(), Some("Microsoft.WindowsCalculator_8wekyb3d8bbwe"));
        assert_eq!(family_name("Bad Name!", MICROSOFT), None);
        assert_eq!(family_name("Studio.Game", " "), None);
    }

    #[test]
    fn only_the_launch_links_upnext_makes_are_handed_out() {
        let link = run_url("Microsoft.Halo_8wekyb3d8bbwe", "Game");
        assert_eq!(link, r"shell:AppsFolder\Microsoft.Halo_8wekyb3d8bbwe!Game");
        assert_eq!(run_link(&link).as_deref(), Some(link.as_str()));
        assert_eq!(run_link("shell:AppsFolder%5CMicrosoft.Halo_8wekyb3d8bbwe%21Game").as_deref(), Some(link.as_str()));
        for other in [
            r"shell:AppsFolder\Microsoft.Halo_8wekyb3d8bbwe!Game ",
            r"shell:AppsFolder\Microsoft.Halo_8wekyb3d8bbwe!Game\..\cmd",
            r"shell:AppsFolder\Microsoft.Halo_8wekyb3d8bbwi!Game",
            r"shell:AppsFolder\Microsoft.Halo!Game",
            r"shell:AppsFolder\..\..\Windows_8wekyb3d8bbwe!Game",
            r"shell:Startup",
            r"shell:System\cmd.exe",
            r"shell:appsfolder\Microsoft.Halo_8wekyb3d8bbwe!Game",
            "steam://run/620",
        ] {
            assert_eq!(run_link(other), None, "{other}");
        }
    }

    #[test]
    fn games_are_read_from_their_config_and_manifest() {
        let dir = tempfile::tempdir().unwrap();
        // The manifest names the application and its identity, and the
        // config's display name is a resource reference, so the folder names it.
        game(
            dir.path(),
            "Forza Horizon 5",
            &config(
                r#"<Identity Name="Microsoft.SunriseBaseGame" Publisher="CN=Old" Version="1.0.0.0"/>"#,
                r#"<ShellVisuals DefaultDisplayName="ms-resource:DisplayName" /><StoreId>9nkx70bbcdrn</StoreId>"#,
            ),
            Some(&format!(
                r#"<Package><Identity Name="Microsoft.SunriseBaseGame" Publisher="{MICROSOFT}" Version="3.0.0.0" />
<Properties><DisplayName>ms-resource:Title</DisplayName></Properties>
<Applications><Application Id="Game" Executable="ForzaHorizon5.exe" /></Applications></Package>"#
            )),
        );
        // No manifest: the config's identity, its PC executable, its name.
        game(
            dir.path(),
            "Ori",
            &config(
                r#"<Identity Publisher='CN=Moon Studios' Name="MoonStudios.Ori" />"#,
                r#"<ShellVisuals DefaultDisplayName="Ori &amp; the Will of the Wisps" PublisherDisplayName="Xbox Game Studios" />"#,
            ),
            None,
        );
        // Not a game: no config. And a config with no identity.
        std::fs::create_dir_all(dir.path().join("Empty").join("Content")).unwrap();
        game(dir.path(), "Broken", "<Game><StoreId>9NBLGGH4R315</StoreId></Game>", None);
        // The same package again, in another folder.
        std::fs::create_dir_all(dir.path().join("Ori copy")).unwrap();
        std::fs::copy(
            dir.path().join("Ori").join("Content").join("MicrosoftGame.config"),
            dir.path().join("Ori copy").join("MicrosoftGame.config"),
        )
        .unwrap();

        let got = installed(&[dir.path().to_path_buf(), dir.path().join("missing")]);
        let ori = format!("MoonStudios.Ori_{}", publisher_id("CN=Moon Studios"));
        assert_eq!(
            got,
            [
                XboxInstall {
                    family: "Microsoft.SunriseBaseGame_8wekyb3d8bbwe".into(),
                    app_id: Some("Game".into()),
                    store_id: Some("9NKX70BBCDRN".into()),
                    name: "Forza Horizon 5".into(),
                },
                XboxInstall {
                    family: ori.clone(),
                    app_id: Some("Game".into()),
                    store_id: None,
                    name: "Ori & the Will of the Wisps".into()
                },
            ]
        );

        let forza = copy(&got[0]);
        assert_eq!(
            (forza.source.as_str(), forza.source_id.as_str(), forza.launcher.as_str(), forza.installed),
            ("xboxone", "9nkx70bbcdrn", "xboxone", true)
        );
        assert_eq!(forza.run_url.as_deref(), Some(r"shell:AppsFolder\Microsoft.SunriseBaseGame_8wekyb3d8bbwe!Game"));
        assert_eq!(forza.store_url.as_deref(), Some("https://apps.microsoft.com/detail/9NKX70BBCDRN"));
        let o = copy(&got[1]);
        assert_eq!((o.source_id.as_str(), o.store_url), (ori.as_str(), None));
    }

    #[test]
    fn an_application_id_windows_wouldnt_take_gives_no_play_link() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = format!(r#"<Identity Name="Studio.Game" Publisher="{MICROSOFT}"/><Application Id="Game!..\cmd"/>"#);
        game(dir.path(), "Game", &config("", r#"<ShellVisuals DefaultDisplayName="Game" />"#), Some(&manifest));
        let got = installed(&[dir.path().to_path_buf()]);
        assert_eq!(got[0].app_id, None);
        assert_eq!(copy(&got[0]).run_url, None);
    }

    #[test]
    fn oversized_files_are_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let padding = " ".repeat(MAX_FILE as usize + 1);
        let id = format!(r#"<Identity Name="Studio.Game" Publisher="{MICROSOFT}"/>"#);
        game(dir.path(), "Game", &config(&id, &padding), None);
        assert!(installed(&[dir.path().to_path_buf()]).is_empty());
    }

    #[test]
    fn gaming_root_files_name_folders_on_their_drive() {
        let utf16 = |names: &[&str]| -> Vec<u8> {
            let mut b = b"RGBX".to_vec();
            b.extend_from_slice(&(names.len() as u32).to_le_bytes());
            for n in names {
                b.extend(n.encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
            }
            b
        };
        assert_eq!(gaming_root_folders(&utf16(&["XboxGames"])), ["XboxGames"]);
        assert_eq!(gaming_root_folders(&utf16(&[r"\Games\Xbox\", "Other"])), [r"Games\Xbox", "Other"]);
        assert!(gaming_root_folders(&utf16(&[r"..\Windows", r"C:\Windows", r"Games\..\..", ""])).is_empty());
        assert!(gaming_root_folders(b"RGBX").is_empty());
        assert!(gaming_root_folders(&[]).is_empty());
    }

    #[test]
    fn tests_never_read_this_computers_drives() {
        assert!(roots().is_empty());
    }
}
