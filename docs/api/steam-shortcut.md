# Add UpNext to Steam

Both routes answer only on this computer, since they read and write Steam's files there.

## GET /api/steam/shortcut

```json
{"target": {"program": "/usr/bin/upnext", "arguments": "app --big-screen"},
 "accounts": [{"account_id": "12345678", "added": false, "layout": false, "layout_error": null, "error": null}],
 "steam_running": false, "unavailable": null}
```

- `target`: what a launcher runs to open UpNext in big screen. Settings shows it, with a copy button, for the launchers UpNext can't add itself to (the Xbox app, Armoury Crate SE, Legion Space). It is null in a container, a build without the app window, or where that program isn't there. In the Flatpak it is `/usr/bin/flatpak` with `run io.github.zytocc.UpNext --big-screen`.
- `accounts`: each Steam account that has signed in on this computer, and whether its library has UpNext's shortcut. `layout` says whether the shortcut has a controller layout. A shortcut without one gets it from Add to Steam, which leaves the shortcut as it is. While `layout` is false, `layout_error` says why the last attempt couldn't write the layouts. `error` says why the account's file can't be read. Both Flatpak and the Steam copy of UpNext have an empty `accounts`.
- `steam_running`: adding waits until Steam is closed.
- `unavailable`: why Add to Steam can't be used here (a container, no app window or program for it, no Steam or no account, the Flatpak, or the Steam copy, which Steam lists already), or null.

## POST /api/steam/shortcut

Adds UpNext as a non-Steam shortcut to every account's library, named UpNext, running `target`, with UpNext's library images. Images already there are left alone.

- Every account's file is read before any is written, so one that can't be read changes none. The first time UpNext changes a file it keeps a copy beside it as `shortcuts.vdf.upnext-backup`. Other shortcuts are untouched, and pressing it twice changes nothing.
- It also gives the shortcut Steam's Gamepad controller layout for each kind of controller, so a pad works as a controller at once rather than moving the mouse. A layout file UpNext can't read is logged and left alone, and the shortcut is added all the same.
- Steam Cloud can replace these files. When the pad still moves the mouse, pick the layout by hand: UpNext's page in Steam, the controller icon, the layout, Templates, Gamepad.

Answers `200` in the shape above, or `409` with why when Steam is running ("Close Steam first"), an account's file can't be read, or it can't be used here.
