# winget

UpNext is in the Windows Package Manager as `Zytoc.UpNext`, so `winget install Zytoc.UpNext` installs it and `winget upgrade Zytoc.UpNext` brings it up to date. winget runs the release's per-user MSI (`contrib/windows/upnext.wxs`) with no window and no administrator prompt, and finds a copy installed from the MSI by hand by its UpgradeCode, so `winget upgrade` updates that one too.

winget's packages live in [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs), three YAML files per version under `manifests/z/Zytoc/UpNext/<version>/`. Each release gets them by a pull request there; Microsoft's checks install the MSI in a sandbox and a moderator merges it, usually within a few days. Until it's merged, winget offers the previous version.

`render.sh` writes the three files for a published release: the MSI's link and SHA-256 from the release, its release date from `CHANGELOG.md`, and, with msitools installed (`msiinfo`), the MSI's ProductCode. The package's description and links are in the script.

## The pull request

With the GitHub CLI signed in as the account that opens the pull request. Once, fork winget-pkgs and make a small checkout of it (`--depth 1` and the sparse checkout keep it to UpNext's folder of a very large repository):

```sh
gh repo fork microsoft/winget-pkgs --clone=false
git clone --depth 1 --filter=blob:none --sparse https://github.com/<you>/winget-pkgs ~/src/winget-pkgs
cd ~/src/winget-pkgs
git sparse-checkout set manifests/z/Zytoc
```

For each release, once its MSI is on the release page and its CHANGELOG section is dated, bring the fork up to date with winget-pkgs, write the files on a branch from it, and open the pull request:

```sh
gh repo sync <you>/winget-pkgs
cd ~/src/winget-pkgs
git fetch --depth 1 origin master
git switch -c upnext-X.Y.Z FETCH_HEAD
~/src/upnext/contrib/winget/render.sh X.Y.Z .
git add manifests/z/Zytoc/UpNext
git commit -m "New version: Zytoc.UpNext version X.Y.Z"
git push -u origin upnext-X.Y.Z
gh pr create -R microsoft/winget-pkgs --head <you>:upnext-X.Y.Z --title "New version: Zytoc.UpNext version X.Y.Z" --body "UpNext X.Y.Z, from https://github.com/zytocc/upnext/releases/tag/vX.Y.Z"
```

The first pull request says "New package" in place of "New version". On a first pull request, Microsoft's CLA bot asks for a comment agreeing to its contributor licence agreement. The checks then label the pull request; one labelled `Needs-Author-Feedback` says in a comment what to change: change the files on the same branch and push.

## Trying the files first

On a Windows computer, with the three files in a folder:

```powershell
winget validate --manifest .\0.1.2
winget settings --enable LocalManifestFiles   # once, in an administrator terminal
winget install --manifest .\0.1.2
```

The install must finish without a window, put UpNext in the Start menu, and show UpNext in Settings › Apps.
