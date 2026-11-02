//! UpNext installed from its Windows package (MSIX), the copy the
//! Microsoft Store installs (`contrib/windows/msix/`). Windows runs it with
//! the package's identity, which changes three things. Its programs are in a
//! folder named for each version under `C:\Program Files\WindowsApps`, so
//! another program (Steam's shortcut) starts it through the package's
//! alias, `upnext-app.exe` in `%LOCALAPPDATA%\Microsoft\WindowsApps`, which
//! stays put across updates. Its writes to `HKCU` go to a registry of the
//! package's own, which Windows doesn't read at sign-in, so Start with
//! Windows is the package's startup task, which the user turns on in
//! Windows' own settings (`autostart`). And Windows moves the files it
//! creates under `%LOCALAPPDATA%` to a folder of the package's own, deleted
//! with it, but changes files that were already there in place: in
//! `%LOCALAPPDATA%\upnext`, which the other copies use, the database would
//! stay put while its journal and backups moved. So the package keeps its
//! files in a folder no other copy uses, [`FOLDER`], all of which Windows
//! moves; the Store copy has a library of its own, which a backup carries
//! over. `contrib/windows/build-msix.ps1` puts `msix-package.txt` beside the
//! programs, which is how UpNext knows.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The marker file beside the programs in the package
/// (`contrib/windows/build-msix.ps1` writes it).
pub const MARKER: &str = "msix-package.txt";

/// The package's folder under `%LOCALAPPDATA%`, in place of `upnext`.
pub const FOLDER: &str = "upnext-store";

/// The package this program is part of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msix {
    /// What another program runs to open the app window: the package's
    /// `upnext-app.exe` alias, which Windows starts with the package's
    /// identity, whichever version is installed.
    pub launcher: PathBuf,
    /// The folder the package keeps its data, settings and cache in, as
    /// the package sees it (`%LOCALAPPDATA%\upnext-store`).
    pub home: PathBuf,
}

/// The package this program is part of, or None for any other copy. Tests
/// never see one, wherever they run.
pub fn get() -> Option<&'static Msix> {
    static FOUND: OnceLock<Option<Msix>> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            if cfg!(test) { None } else { std::env::current_exe().ok().and_then(|exe| detect(&exe, std::env::var_os("LOCALAPPDATA"))) }
        })
        .as_ref()
}

/// The package whose program is `exe`, by the marker beside it, with its
/// alias under `local_app_data`.
fn detect(exe: &Path, local_app_data: Option<OsString>) -> Option<Msix> {
    if !exe.parent()?.join(MARKER).is_file() {
        return None;
    }
    let local = PathBuf::from(local_app_data.filter(|l| !l.is_empty())?);
    Some(Msix { launcher: local.join("Microsoft").join("WindowsApps").join("upnext-app.exe"), home: local.join(FOLDER) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_beside_the_program_makes_it_the_package() {
        let install = tempfile::tempdir().unwrap();
        let exe = install.path().join("upnext-app.exe");
        let local = || Some(OsString::from("/home/you/AppData/Local"));
        assert_eq!(detect(&exe, local()), None, "no marker: installed some other way");

        std::fs::write(install.path().join(MARKER), "installed from the package").unwrap();
        let local_dir = Path::new("/home/you/AppData/Local");
        let alias = local_dir.join("Microsoft").join("WindowsApps").join("upnext-app.exe");
        let found = detect(&exe, local()).unwrap();
        assert_eq!(found.launcher, alias, "Steam starts the alias, not the versioned folder");
        assert_eq!(found.home, local_dir.join("upnext-store"), "not the other copies' folder");
        assert_eq!(detect(&exe, None), None, "no LOCALAPPDATA, no alias to name");
        assert_eq!(detect(&exe, Some(OsString::new())), None);

        // A folder named like the marker isn't one.
        let other = tempfile::tempdir().unwrap();
        std::fs::create_dir(other.path().join(MARKER)).unwrap();
        assert_eq!(detect(&other.path().join("upnext.exe"), local()), None);
    }

    #[test]
    fn tests_never_run_as_the_package() {
        assert_eq!(get(), None);
    }
}
