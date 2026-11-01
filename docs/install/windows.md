# Windows

Download `upnext-x86_64-pc-windows-msvc.msi` from the [latest release](https://github.com/zytocc/upnext/releases/latest) and run it. It needs Windows 10 or 11 on x64. It installs UpNext for you alone in `%LOCALAPPDATA%\Programs\UpNext`, without administrator rights, and puts `upnext` on your `PATH` for terminals opened afterwards.

Two Start menu entries are added: "UpNext", which opens the app window, and "UpNext (big screen)", which opens it full screen for a controller ([Big screen](../big-screen.md)). The zip beside the installer has the same two programs, `upnext.exe` and `upnext-app.exe`, to run without installing.

To update, run a newer installer. To uninstall, use Settings › Apps; your library in `%LOCALAPPDATA%\upnext` stays.

## SmartScreen and checksums

UpNext's Windows builds aren't code-signed ([Code signing policy](../../README.md#code-signing-policy)), so Windows is wary of them at first:

- SmartScreen says "Windows protected your PC". Choose **More info**, then **Run anyway**.
- With Smart App Control on, Windows may block the installer outright, with no way to run it anyway.
- To check the file, compare its SHA-256 with the `.sha256` file beside it on the release page. In PowerShell, `(Get-FileHash .\upnext-x86_64-pc-windows-msvc.msi).Hash.ToLower()` must print the first word of that file.

## The icon by the clock

While the app runs, UpNext's icon sits by the clock (Windows may tuck it under the arrow). Closing the window leaves UpNext running there, so the daily sync and other jobs keep their schedules. Click the icon to open the window again, or choose Quit in its menu. To make closing the window stop UpNext, turn off "Keep running in the background" in Settings › Windows.

"Start with Windows", in the same place and in the icon's menu, starts UpNext when you sign in, with only the icon. It's off until you turn it on.

## WebView2

The app window needs Microsoft's WebView2 Runtime, which Windows 11 includes. If it's missing, UpNext offers Microsoft's download page.

To build it yourself, see [Building from source](source.md).
