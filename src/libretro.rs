//! Box art for games added by hand, from libretro's thumbnail server (the
//! covers RetroArch shows). No key. Each system's `Named_Boxarts` folder is
//! a plain file listing; a game's cover is the file named after it, with the
//! region in brackets: "Demon's Souls (USA).png". The images are linked, as
//! every cover is, never copied.

use crate::http::{Backoff, RateLimiter, base_url, send_with_backoff};
use crate::names::name_key;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// The server, which the covers' links always name.
pub const BASE: &str = "https://thumbnails.libretro.com";

/// The systems "where it's from" names, by the words that name them: an
/// emulator, the console, or its short name. Longer phrases come first, so
/// "wii u" is read before "wii" and "playstation 2" before "playstation".
/// The system names are libretro's folders. Switch games have none.
const SYSTEMS: &[(&str, &[&str])] = &[
    ("xbox 360", &["Microsoft - Xbox 360"]),
    ("xenia", &["Microsoft - Xbox 360"]),
    ("original xbox", &["Microsoft - Xbox"]),
    ("xemu", &["Microsoft - Xbox"]),
    ("playstation portable", &["Sony - PlayStation Portable"]),
    ("playstation vita", &["Sony - PlayStation Vita"]),
    ("ps vita", &["Sony - PlayStation Vita"]),
    ("playstation 4", &["Sony - PlayStation 4"]),
    ("playstation 3", &["Sony - PlayStation 3", "Sony - PlayStation 3 (Downloadable)"]),
    ("playstation 2", &["Sony - PlayStation 2"]),
    ("playstation 1", &["Sony - PlayStation"]),
    ("playstation", &["Sony - PlayStation"]),
    ("shadps4", &["Sony - PlayStation 4"]),
    ("rpcs3", &["Sony - PlayStation 3", "Sony - PlayStation 3 (Downloadable)"]),
    ("pcsx2", &["Sony - PlayStation 2"]),
    ("aethersx2", &["Sony - PlayStation 2"]),
    ("nethersx2", &["Sony - PlayStation 2"]),
    ("duckstation", &["Sony - PlayStation"]),
    ("swanstation", &["Sony - PlayStation"]),
    ("epsxe", &["Sony - PlayStation"]),
    ("ppsspp", &["Sony - PlayStation Portable"]),
    ("vita3k", &["Sony - PlayStation Vita"]),
    ("ps4", &["Sony - PlayStation 4"]),
    ("ps3", &["Sony - PlayStation 3", "Sony - PlayStation 3 (Downloadable)"]),
    ("ps2", &["Sony - PlayStation 2"]),
    ("ps1", &["Sony - PlayStation"]),
    ("psx", &["Sony - PlayStation"]),
    ("psp", &["Sony - PlayStation Portable"]),
    ("vita", &["Sony - PlayStation Vita"]),
    ("game boy advance", &["Nintendo - Game Boy Advance"]),
    ("game boy color", &["Nintendo - Game Boy Color"]),
    ("game boy", &["Nintendo - Game Boy"]),
    ("super nintendo", &["Nintendo - Super Nintendo Entertainment System"]),
    ("super famicom", &["Nintendo - Super Nintendo Entertainment System"]),
    ("nintendo 64", &["Nintendo - Nintendo 64"]),
    ("nintendo 3ds", &["Nintendo - Nintendo 3DS", "Nintendo - Nintendo 3DS (Digital)"]),
    ("nintendo ds", &["Nintendo - Nintendo DS"]),
    ("wii u", &["Nintendo - Wii U"]),
    ("dolphin", &["Nintendo - GameCube", "Nintendo - Wii"]),
    ("gamecube", &["Nintendo - GameCube"]),
    ("wii", &["Nintendo - Wii"]),
    ("cemu", &["Nintendo - Wii U"]),
    ("citra", &["Nintendo - Nintendo 3DS", "Nintendo - Nintendo 3DS (Digital)"]),
    ("lime3ds", &["Nintendo - Nintendo 3DS", "Nintendo - Nintendo 3DS (Digital)"]),
    ("azahar", &["Nintendo - Nintendo 3DS", "Nintendo - Nintendo 3DS (Digital)"]),
    ("3ds", &["Nintendo - Nintendo 3DS", "Nintendo - Nintendo 3DS (Digital)"]),
    ("melonds", &["Nintendo - Nintendo DS"]),
    ("desmume", &["Nintendo - Nintendo DS"]),
    ("nds", &["Nintendo - Nintendo DS"]),
    ("mgba", &["Nintendo - Game Boy Advance", "Nintendo - Game Boy Color", "Nintendo - Game Boy"]),
    ("gba", &["Nintendo - Game Boy Advance"]),
    ("gbc", &["Nintendo - Game Boy Color"]),
    ("snes9x", &["Nintendo - Super Nintendo Entertainment System"]),
    ("bsnes", &["Nintendo - Super Nintendo Entertainment System"]),
    ("snes", &["Nintendo - Super Nintendo Entertainment System"]),
    ("mupen64plus", &["Nintendo - Nintendo 64"]),
    ("project64", &["Nintendo - Nintendo 64"]),
    ("simple64", &["Nintendo - Nintendo 64"]),
    ("n64", &["Nintendo - Nintendo 64"]),
    ("famicom", &["Nintendo - Nintendo Entertainment System"]),
    ("nestopia", &["Nintendo - Nintendo Entertainment System"]),
    ("fceux", &["Nintendo - Nintendo Entertainment System"]),
    ("nes", &["Nintendo - Nintendo Entertainment System"]),
    ("dreamcast", &["Sega - Dreamcast"]),
    ("flycast", &["Sega - Dreamcast"]),
    ("redream", &["Sega - Dreamcast"]),
    ("saturn", &["Sega - Saturn"]),
    ("mega drive", &["Sega - Mega Drive - Genesis"]),
    ("megadrive", &["Sega - Mega Drive - Genesis"]),
    ("genesis", &["Sega - Mega Drive - Genesis"]),
    ("master system", &["Sega - Master System - Mark III"]),
    ("game gear", &["Sega - Game Gear"]),
    ("neo geo", &["SNK - Neo Geo"]),
    ("pc engine", &["NEC - PC Engine - TurboGrafx 16"]),
    ("turbografx", &["NEC - PC Engine - TurboGrafx 16"]),
    ("atari 2600", &["Atari - 2600"]),
];

/// The libretro systems a game from `from` ("RPCS3", "my PS2") may have
/// box art in, most likely first; none for a place that names no system.
pub fn systems_for(from: &str) -> Vec<&'static str> {
    let words = format!(" {} ", name_key(from));
    let mut out: Vec<&'static str> = vec![];
    let mut rest = words.clone();
    for (phrase, systems) in SYSTEMS {
        let needle = format!(" {phrase} ");
        if rest.contains(&needle) {
            // The words a longer phrase took aren't read again ("wii u" isn't also "wii").
            rest = rest.replace(&needle, " ");
            for s in *systems {
                if !out.contains(s) {
                    out.push(s);
                }
            }
        }
    }
    out.truncate(3);
    out
}

/// What a box art name matches by: the name without its brackets ("(USA)",
/// "[b]"), the article libretro puts last put first again, as `name_key`
/// reads it. "Legend of Zelda, The - Ocarina of Time (USA)" and "The Legend
/// of Zelda: Ocarina of Time" match.
pub fn match_key(name: &str) -> String {
    let mut plain = String::new();
    let mut depth = 0;
    for c in name.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = (depth - 1).max(0),
            _ if depth == 0 => plain.push(c),
            _ => {}
        }
    }
    let plain = plain.trim();
    for article in ["The", "A", "An"] {
        let mid = format!(", {article} - ");
        if let Some((head, tail)) = plain.split_once(&mid) {
            return name_key(&format!("{article} {head} - {tail}"));
        }
    }
    name_key(plain)
}

/// How much a box art's brackets make it the one to show: an English
/// release first, then the rest; never a demo or a prototype.
fn rank(name: &str) -> Option<u8> {
    let lower = name.to_lowercase();
    if ["(demo", "(beta", "(proto", "(sample", "(kiosk", "(preview"].iter().any(|w| lower.contains(w)) {
        return None;
    }
    Some(if lower.contains("(usa") {
        0
    } else if lower.contains("(world") {
        1
    } else if lower.contains("(europe") || lower.contains("(uk") {
        2
    } else if lower.contains("(japan") {
        4
    } else {
        3
    })
}

/// The box art name in `names` that is the game called `name`, the most
/// fitting release of it; None when no name matches exactly.
pub fn best_match<'a>(name: &str, names: &'a [String]) -> Option<&'a str> {
    let want = match_key(name);
    if want.is_empty() {
        return None;
    }
    names.iter().filter(|n| match_key(n) == want).filter_map(|n| rank(n).map(|r| (r, n.len(), n))).min().map(|(_, _, n)| n.as_str())
}

/// The link to `name`'s box art in `system`.
pub fn boxart_url(system: &str, name: &str) -> String {
    let mut url = reqwest::Url::parse(BASE).expect("a valid base");
    if let Ok(mut path) = url.path_segments_mut() {
        path.push(system).push("Named_Boxarts").push(&format!("{name}.png"));
    }
    url.to_string()
}

/// The box art names in a folder's listing: every `.png` it links to,
/// decoded, without the extension.
pub fn parse_listing(html: &str) -> Vec<String> {
    let mut out = vec![];
    for part in html.split("href=\"").skip(1) {
        let Some(href) = part.split('"').next() else { continue };
        if href.contains('/') || href.starts_with('?') {
            continue;
        }
        let decoded = decode(href);
        if let Some(stem) = decoded.strip_suffix(".png") {
            out.push(stem.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// `%XX` escapes decoded; a malformed one is kept as it is.
fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(b) = std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub struct LibretroClient {
    http: Client,
    base: String,
    limiter: RateLimiter,
}

impl LibretroClient {
    /// `base` stands in for the server (tests pass a local mock server).
    pub fn new(http: Client, base: Option<&str>) -> Self {
        Self { http, base: base_url(BASE, base), limiter: RateLimiter::new("libretro", Duration::from_millis(1000)) }
    }

    /// The box art names libretro has for `system`.
    pub async fn boxarts(&self, system: &str) -> Result<Vec<String>> {
        let mut url = reqwest::Url::parse(&format!("{}/", self.base)).context("the libretro server's address")?;
        if let Ok(mut path) = url.path_segments_mut() {
            path.pop_if_empty().push(system).push("Named_Boxarts").push("");
        }
        let backoff = Backoff { attempts: 3, base: Duration::from_secs(5), max: Duration::from_secs(30), ..Default::default() };
        let resp = send_with_backoff(&self.limiter, &backoff, || self.http.get(url.clone()).timeout(Duration::from_secs(60))).await?;
        let html = resp.text().await.context("libretro's listing")?;
        Ok(parse_listing(&html))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn where_its_from_names_the_system() {
        assert_eq!(systems_for("RPCS3"), ["Sony - PlayStation 3", "Sony - PlayStation 3 (Downloadable)"]);
        assert_eq!(systems_for("my PS2"), ["Sony - PlayStation 2"]);
        assert_eq!(systems_for("Wii U (Cemu)"), ["Nintendo - Wii U"]);
        assert_eq!(systems_for("Wii"), ["Nintendo - Wii"]);
        assert_eq!(systems_for("Dolphin"), ["Nintendo - GameCube", "Nintendo - Wii"]);
        assert_eq!(systems_for("PlayStation 2"), ["Sony - PlayStation 2"]);
        assert!(systems_for("Meta Quest").is_empty());
        assert!(systems_for("RetroArch").is_empty(), "a frontend names no system");
        assert!(systems_for("Xbox").is_empty(), "the Xbox app's PC games aren't the first Xbox's");
        assert!(systems_for("Ryujinx").is_empty(), "libretro has no Switch art");
    }

    #[test]
    fn a_name_matches_libretros_way_of_writing_it() {
        let names: Vec<String> = [
            "Demon's Souls (Europe)",
            "Demon's Souls (USA)",
            "Demon's Souls (USA) (Demo)",
            "Legend of Zelda, The - Ocarina of Time (USA)",
            "Atelier Escha _ Logy - Alchemists of the Dusk Sky (Japan)",
            "Castlevania - Lords of Shadow (USA)",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(best_match("Demon's Souls", &names), Some("Demon's Souls (USA)"));
        assert_eq!(best_match("The Legend of Zelda: Ocarina of Time", &names), Some("Legend of Zelda, The - Ocarina of Time (USA)"));
        assert_eq!(best_match("Atelier Escha & Logy: Alchemists of the Dusk Sky", &names).map(|n| n.contains("Japan")), Some(true));
        assert_eq!(best_match("Castlevania: Lords of Shadow", &names), Some("Castlevania - Lords of Shadow (USA)"));
        assert_eq!(best_match("Castlevania", &names), None, "only the whole name matches");
    }

    #[test]
    fn a_listing_gives_the_names_and_a_cover_its_link() {
        let html = r#"<a href="?C=N;O=D">Name</a> <a href="/">Parent</a>
            <a href="Demon's%20Souls%20(USA).png">Demon's Souls (USA).png</a>
            <a href="Atelier%20Escha%20_%20Logy%20(Japan).png">x</a> <a href="notes.txt">notes</a>"#;
        assert_eq!(parse_listing(html), ["Atelier Escha _ Logy (Japan)", "Demon's Souls (USA)"]);
        assert_eq!(
            boxart_url("Sony - PlayStation 3", "Demon's Souls (USA)"),
            "https://thumbnails.libretro.com/Sony%20-%20PlayStation%203/Named_Boxarts/Demon's%20Souls%20(USA).png"
        );
    }

    #[tokio::test]
    async fn the_client_reads_a_systems_folder() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let s = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/Sony%20-%20PlayStation%202/Named_Boxarts/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"<a href="Okami%20(USA).png">Okami (USA).png</a>"#))
            .mount(&s)
            .await;
        let names = LibretroClient::new(Client::new(), Some(&s.uri())).boxarts("Sony - PlayStation 2").await.unwrap();
        assert_eq!(names, ["Okami (USA)"]);
    }
}
