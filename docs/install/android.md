# Android

The Android app uses UpNext on your computer from a phone, a tablet or an Android TV (Android 8 or later). UpNext itself runs on the computer; the app finds it, pairs with it and shows its UI. A release carries it as `upnext-android-<version>.apk`, with its `.sha256`.

1. On the computer, turn on **Allow other devices** (Settings › Devices and API).
2. On the phone, download the APK from a [release](https://github.com/zytocc/upnext/releases) and open it. The first time, Android asks you to let the app you downloaded it with (the browser or Files) install apps. To check the file first: `sha256sum -c upnext-android-<version>.apk.sha256`.
3. Open the app. It lists the computers on your network that run UpNext; tap yours. The computer shows a four-digit number; check the phone shows the same one and choose **Allow** there ([Pairing](../other-devices.md#pairing)). The phone goes straight to UpNext from then on.

Away from home, or on a network that hides the computer, choose **Add by address** (for example `https://den.tailnet.ts.net`), or **Scan the QR code** from Settings › Devices and API, Add a device, which pairs with the code it carries. "Type a code instead" takes the same six digits.

The app keeps several computers. **Forget** removes one from the phone and from that computer's devices. A new phone asks again.

Games start on the computer. On an Android TV, UpNext is in the launcher's apps and opens in [big screen](../big-screen.md), for the remote or a gamepad.

To update, install a newer APK over the old one. To uninstall, remove the app as any other.
