# Self-hosted CI runners

`ci`, `windows` and `docker` run on GitHub's runners unless a repository variable names a self-hosted runner. This page installs a runner on an Arch-based Linux machine and on a Windows 11 desktop and points the workflows at them. Run every command as the maintainer, on the machine named in its section.

## What runs where

Each workflow reads one repository variable. Unset, the job runs on GitHub's runner. Set to a label, it runs on the self-hosted runner that carries that label.

| Workflow | Jobs | Variable | Value | Unset |
|---|---|---|---|---|
| `ci` | `rust`, `web`, `verdicts-service`, `e2e` | `UPNEXT_RUNNER_LINUX` | `upnext-linux` | `ubuntu-latest` |
| `windows` | `rust` | `UPNEXT_RUNNER_WINDOWS` | `upnext-windows` | `windows-latest` |
| `docker` | `image` | `UPNEXT_RUNNER_DOCKER` | `upnext-linux`, on a runner set up as in "Rootless Docker" | `ubuntu-latest` |
| `android` | `build` | `UPNEXT_ANDROID_CI` = `on`, then `UPNEXT_RUNNER_LINUX` as for `ci` | needs JDK 17 and the Android SDK (`ANDROID_HOME`) on the runner | off |
| `flatpak` | `build` | `UPNEXT_FLATPAK_CI` = `on` | GitHub's runner only (flatpak-builder needs a privileged container) | off |
| `release` | all | none | always GitHub's runners | |

On a self-hosted runner every job is skipped for a pull request from a fork. Pushes, manual runs and pull requests from branches of this repository (Dependabot's included) run.

## Rules

- The `UPNEXT_RUNNER_*` variables are for a repository whose pull requests come only from its maintainers. Never set them where outside pull requests run: such a pull request runs its own copy of the workflow files, so it could delete the fork check and run anything on these machines.
- Leave "Run workflows from fork pull requests" off (Settings, Actions, General).
- Each runner runs as an account that can't read the maintainer's files: a dedicated `github-runner` user on Linux, Network Service on Windows. CI builds third-party code.
- No runner account is in the `docker` group. The Linux runner's Docker, if it has one, is rootless.
- One runner per machine: the `e2e` job uses ports 8798 and 8799.

## A registration token

A runner registers with a token that lasts one hour. With `gh` signed in as an admin of the repository:

```sh
R=zytocc/upnext
gh api -X POST repos/$R/actions/runners/registration-token --jq .token
```

Without `gh`, Settings, Actions, Runners, "New self-hosted runner" shows a token (after `--token`) and the current runner version and SHA-256.

## Linux runner (Arch-based)

### 1. Packages

Update the system first (`sudo pacman -Syu`). Then:

```sh
# The runner (.NET needs ICU and Kerberos), and UpNext's desktop feature (WebKitGTK 4.1, GTK 3).
sudo pacman -S --needed base-devel git pkgconf jq icu krb5 webkit2gtk-4.1 gtk3
# Playwright's Chromium libraries and fonts.
sudo pacman -S --needed nss nspr alsa-lib at-spi2-core libcups dbus libdrm libxkbcommon \
  libxcomposite libxdamage libxext libxfixes libxrandr mesa pango cairo ttf-liberation
```

### 2. An account for the runner

```sh
sudo useradd --create-home --shell /bin/bash github-runner
ls -ld /home/*    # your own home should read drwx------ (or drwx--x---), so the runner can't read it
```

### 3. Rust for that account

```sh
sudo -iu github-runner bash -c \
  'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable -c clippy,rustfmt'
```

Node 24, Playwright's Chromium and cargo-deny need nothing here: the workflows download them on the first run (Node and Chromium are cached in the runner's account).

### 4. The runner, as a service

As `github-runner`, download the runner and check it against the release's SHA-256:

```sh
sudo -iu github-runner
rel=$(curl -fsSL https://api.github.com/repos/actions/runner/releases/latest)
V=$(jq -r .tag_name <<<"$rel" | sed 's/^v//')
SHA=$(jq -r .body <<<"$rel" | sed -n 's/.*<!-- BEGIN SHA linux-x64 -->\([0-9a-f]\{64\}\)<!-- END SHA linux-x64 -->.*/\1/p')
echo "runner $V, sha256 $SHA"    # both must be set; or copy them from the "New self-hosted runner" page
mkdir -p ~/actions-runner && cd ~/actions-runner
curl -fsSL -o runner.tar.gz "https://github.com/actions/runner/releases/download/v$V/actions-runner-linux-x64-$V.tar.gz"
echo "$SHA  runner.tar.gz" | sha256sum -c -
tar xzf runner.tar.gz && rm runner.tar.gz
./config.sh --url https://github.com/zytocc/upnext --token <TOKEN> \
  --name upnext-linux-1 --labels upnext-linux --work _work --unattended
exit
```

Then install it as a system service that runs as `github-runner` and starts at boot:

```sh
cd /home/github-runner/actions-runner
sudo ./svc.sh install github-runner
sudo ./svc.sh start
sudo ./svc.sh status    # active (running), "Listening for Jobs" in the log
```

The unit is `actions.runner.zytocc-upnext.upnext-linux-1.service`; `journalctl -u 'actions.runner.*' -f` follows it. The runner updates itself.

### 5. Rootless Docker (optional, for the `docker` workflow)

The `docker` workflow builds the amd64 and arm64 images with buildx (arm64 under QEMU) and pushes them to GHCR. Here Docker runs rootless, so nothing a CI job runs gets root. Skip this section to leave `docker` on GitHub's runners.

**a. The system's part** (your own account, with sudo):

```sh
sudo pacman -S --needed docker docker-buildx rootlesskit slirp4netns qemu-user-static qemu-user-static-binfmt
command -v iptables || sudo pacman -S --needed iptables-nft
# The rootless parts Arch's package leaves out (AUR; read its PKGBUILD first).
git clone https://aur.archlinux.org/docker-rootless-extras.git /tmp/docker-rootless-extras
(cd /tmp/docker-rootless-extras && less PKGBUILD && makepkg -si)
sudo sysctl --system
id -nG github-runner | grep -qw docker && echo "remove github-runner from the docker group" || echo ok
# QEMU for arm64, registered with the F flag.
sudo systemctl restart systemd-binfmt.service
grep -E '^(enabled|flags)' /proc/sys/fs/binfmt_misc/qemu-aarch64    # "enabled" and flags containing F
# Subordinate ids, if useradd didn't add them.
grep github-runner /etc/subuid /etc/subgid \
  || sudo usermod --add-subuids 231072-296607 --add-subgids 231072-296607 github-runner
# Start the runner's own systemd at boot, without a login.
sudo loginctl enable-linger github-runner
# nf_tables (ip_tables if `iptables --version` says legacy).
echo nf_tables | sudo tee /etc/modules-load.d/docker-rootless.conf && sudo modprobe nf_tables
```

**b. The daemon** (as `github-runner`, in a real login session so `systemctl --user` works; `sudo -iu` doesn't give one):

```sh
sudo machinectl shell github-runner@ /bin/bash
systemctl --user enable --now docker.socket
export DOCKER_HOST=unix://$XDG_RUNTIME_DIR/docker.sock
docker info --format '{{.SecurityOptions}}'        # includes name=rootless
docker buildx version                              # prints a version
docker run --rm --platform linux/arm64 alpine uname -m    # aarch64
exit
```

If `docker info` fails, `journalctl --user -u docker.service` (as `github-runner`) names what's missing: usually the subordinate ids, `nf_tables` or `slirp4netns`.

**c. Point the runner at it.** The runner reads extra environment variables from `.env` in its folder at start:

```sh
cd /home/github-runner/actions-runner
echo "DOCKER_HOST=unix:///run/user/$(id -u github-runner)/docker.sock" | sudo -u github-runner tee -a .env
sudo ./svc.sh stop && sudo ./svc.sh start
```

### 6. A machine without working IPv6

On a machine whose name lookups return IPv6 addresses but whose IPv6 traffic is dropped (for example behind some VPNs), the `e2e` job hangs at `npx playwright install chromium`. Forbid IPv6 sockets to the runner service, so the downloader falls back to IPv4:

```sh
d=/etc/systemd/system/actions.runner.zytocc-upnext.upnext-linux-1.service.d
sudo mkdir -p $d
printf '[Service]\nRestrictAddressFamilies=~AF_INET6\n' | sudo tee $d/no-ipv6.conf
sudo systemctl daemon-reload
sudo systemctl restart actions.runner.zytocc-upnext.upnext-linux-1.service
```

The unit's name is `actions.runner.<owner>-<repository>.<runner name>.service`; `systemctl list-units 'actions.runner.*'` shows it.

## Windows runner (Windows 11 Home)

Run everything below in PowerShell as Administrator.

### 1. Tools

```powershell
winget install -e --id Git.Git                  # skip if Git for Windows is installed
winget install -e --id Microsoft.PowerShell     # PowerShell 7 (pwsh), which the workflow's scripts use
winget install -e --id NASM.NASM                # aws-lc-sys assembles its x86_64 code with NASM
winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override `
  "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"   # MSVC and the Windows SDK
```

Install PowerShell 7 and NASM for all users (winget's `--scope machine`): the runner's account can't see a per-user install. Check that `C:\Program Files\PowerShell\7\pwsh.exe` and `C:\Program Files\NASM\nasm.exe` exist.

Put Git's `bin` folder and NASM on the machine's `PATH`, ahead of `System32`, so the workflows' `shell: bash` steps find Git Bash and not WSL's `bash.exe`:

```powershell
$add = 'C:\Program Files\Git\bin', 'C:\Program Files\NASM'
$path = [Environment]::GetEnvironmentVariable('Path', 'Machine') -split ';'
[Environment]::SetEnvironmentVariable('Path', (($add + $path) | Where-Object { $_ } | Select-Object -Unique) -join ';', 'Machine')
```

Restart Windows afterwards: a service takes its environment when Windows starts. Rust needs no install: the first run installs rustup and the stable toolchain for the runner's account.

### 2. The runner, as a service

`gh` must be signed in as an admin of the repository (or paste a token from the "New self-hosted runner" page into `$token`).

```powershell
$R = 'zytocc/upnext'
$token = gh api -X POST "repos/$R/actions/runners/registration-token" --jq .token
New-Item -ItemType Directory -Force C:\actions-runner | Out-Null
Set-Location C:\actions-runner
$rel = Invoke-RestMethod https://api.github.com/repos/actions/runner/releases/latest
$V = $rel.tag_name.TrimStart('v')
$sha = [regex]::Match($rel.body, '<!-- BEGIN SHA win-x64 -->([0-9a-f]{64})<!-- END SHA win-x64 -->').Groups[1].Value
if (-not $sha) { throw 'no SHA-256 for win-x64 in the release notes; copy it from the "New self-hosted runner" page' }
Invoke-WebRequest "https://github.com/actions/runner/releases/download/v$V/actions-runner-win-x64-$V.zip" -OutFile runner.zip
if ((Get-FileHash runner.zip -Algorithm SHA256).Hash -ne $sha) { throw 'runner.zip does not match the release SHA-256' }
Expand-Archive runner.zip -DestinationPath . ; Remove-Item runner.zip
.\config.cmd --url "https://github.com/$R" --token $token --name upnext-windows-1 --labels upnext-windows `
  --work _work --runasservice --unattended
```

This installs the service `actions.runner.zytocc-upnext.upnext-windows-1` running as `NT AUTHORITY\NETWORK SERVICE`, started at boot. Check it:

```powershell
Get-Service actions.runner.* | Format-Table Name, Status, StartType
```

A job waits while the PC sleeps and starts when it wakes.

## Turning the workflows over to the runners

Once both runners are online:

```sh
R=zytocc/upnext
gh variable set UPNEXT_RUNNER_LINUX --body upnext-linux --repo $R
gh variable set UPNEXT_RUNNER_WINDOWS --body upnext-windows --repo $R
gh variable set UPNEXT_RUNNER_DOCKER --body upnext-linux --repo $R    # only after "Rootless Docker" above
gh workflow run ci --ref main --repo $R
gh workflow run windows --ref main --repo $R
gh run list --repo $R --limit 5
```

The value is the label alone, with no quotes or brackets. A label no online runner carries leaves the job queued.

## Checking a runner is online

```sh
gh api repos/zytocc/upnext/actions/runners \
  --jq '.runners[] | {id, name, os, status, busy, labels: [.labels[].name]}'
```

`status` is `online` and the labels include `upnext-linux` or `upnext-windows`. On the machine, `sudo ./svc.sh status` (Linux) or `Get-Service actions.runner.*` (Windows) shows the service. A job's "Set up job" step names its runner.

## Removing a runner

Linux, in `/home/github-runner/actions-runner`:

```sh
sudo ./svc.sh stop && sudo ./svc.sh uninstall
sudo -u github-runner ./config.sh remove --token <REMOVE_TOKEN>
sudo userdel -r github-runner    # also deletes its toolchains, caches and build folder
```

Windows, as Administrator, in `C:\actions-runner` (this also removes the service):

```powershell
.\config.cmd remove --token (gh api -X POST repos/zytocc/upnext/actions/runners/remove-token --jq .token)
Set-Location \; Remove-Item C:\actions-runner -Recurse -Force
```

`<REMOVE_TOKEN>` comes from `gh api -X POST repos/zytocc/upnext/actions/runners/remove-token --jq .token`. If the machine is gone, run `gh api -X DELETE repos/zytocc/upnext/actions/runners/<id>`, with the id from the list above.

## Going back to GitHub's runners

Delete the variables; the next run uses GitHub's runners:

```sh
for v in UPNEXT_RUNNER_LINUX UPNEXT_RUNNER_WINDOWS UPNEXT_RUNNER_DOCKER; do gh variable delete $v --repo zytocc/upnext; done
gh variable list --repo zytocc/upnext    # no UPNEXT_RUNNER_* left
```

Then remove both runners as above.

## When something fails

- **A job stays queued:** no online runner has its label. Check the list above, the variable's value, and that the machine is awake.
- **`bash` not found, or WSL starts, on Windows:** Git's `bin` isn't ahead of `System32` on the machine `PATH`, or Windows wasn't restarted since.
- **`link.exe` not found, or NASM errors, on Windows:** the Build Tools' C++ workload or NASM is missing. Install it and run `Restart-Service actions.runner.*`.
- **`e2e` hangs at `npx playwright install chromium` on Linux:** add the drop-in in section 6.
- **A Playwright test runs out of time in `e2e`:** mark a long test `test.slow()`, as the viewport test is.
- **Chromium won't start in `e2e` on Linux:** `sudo find ~github-runner/.cache/ms-playwright -type f -name '*headless*shell' -exec ldd {} \; | grep 'not found'` names the missing library; install its package.
- **`docker` can't reach the daemon:** as `github-runner` (through `machinectl shell`), check `systemctl --user status docker.socket docker.service` and that `loginctl show-user github-runner --property=Linger` says `yes`. Check that `.env` has the `DOCKER_HOST` line, and restart the runner after changing it.
- **`exec format error` in the arm64 smoke test:** QEMU isn't registered; check `/proc/sys/fs/binfmt_misc/qemu-aarch64` as in section 5a.
- **Disk filling up:** each runner keeps one build folder beside its checkout (`_work/upnext/cargo-target`), which the workflow deletes past 30 GB. You can delete it any time between runs.
