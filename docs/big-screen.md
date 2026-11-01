# Big screen

Big screen is the same UpNext with large type and one focus you move with a gamepad, the arrow keys or a TV remote, for a Steam Deck, a Windows handheld or a computer on the TV. Turn it on in Settings › Big screen ("Switch to big screen"), by adding `?big=1` to the address, or when UpNext offers it after you first press a gamepad button. Each browser remembers it.

| Pad | Keys | Does |
| --- | --- | --- |
| D-pad or left stick | Arrows | Move |
| A | Enter | Choose |
| B | Esc or Backspace | Back, closing a menu first |
| X | M | The card's or game's menu |
| Y | / | Change the request |
| LB, RB | [ ] | The previous or next page |
| LT, RT | Page Up, Page Down | Page through a long list |
| Menu | S | Settings |
| View | ? | The buttons |

The hints name the buttons of the pad in hand. If the pad isn't noticed, press any button once. A request made in big screen counts as on a controller: games Steam lists with controller support come first, unless the routine you choose says what you play on.

## The app window in big screen

`upnext app --big-screen` opens the app full screen in big screen. On Windows that's the Start menu's "UpNext (big screen)"; on Linux, the launcher entry's "Big screen" action. F11 leaves full screen and goes back, and Close UpNext, in the button help or big screen's Settings, closes it. A window opened this way ends when it closes, with no icon left by the clock.

An app window opened the usual way goes full screen when it switches to big screen, and leaving big screen gives back its size.

On Linux, an Xbox controller on Microsoft's wireless adapter (the `xone` driver) gets a layout added to `~/.config/libmanette/gamecontrollerdb`. To remove it, delete the line that starts `# Added by UpNext` and the line under it.

## A first run with a controller

When setup opens in big screen, it runs the steps that need no typing: it reads the games Steam, Heroic and Galaxy keep on the computer, then hands the rest (an AI's key, a store's sign-in) to your phone or computer with a QR code that opens setup there, paired. It ends with [Add to Steam](#add-to-steam) and a short tour of the buttons.

## Steam

### Add to Steam

Settings › Big screen, on the computer UpNext runs on, has **Add to Steam**. It puts UpNext in the Steam library of every account on that computer, under Non-Steam, opening in big screen, so Big Picture, a Deck's Gaming mode or a handheld's Steam can start it. Close Steam first, because Steam rewrites its shortcuts as it exits.

UpNext backs up Steam's shortcuts file first and leaves your other shortcuts alone. It also gives the shortcut Steam's Gamepad controller layout. If the pad still moves the mouse, pick the layout by hand on UpNext's page in Steam: the controller icon, the layout, Templates, Gamepad.

The same section shows the program and arguments to give a handheld's own launcher (the Xbox app, Armoury Crate SE, Legion Space), with a copy button. In the Flatpak, Add to Steam shows how to add UpNext by hand ([Flatpak and Steam Deck](install/flatpak.md#on-a-steam-deck)).

Steam's Gaming mode on a Deck, Big Picture, and Steam on a Deck in Desktop mode open UpNext in big screen by themselves.

### Playing from Steam's big screen

When Big Picture or a Deck's Gaming mode started UpNext, pressing Play closes UpNext a couple of seconds after the game's launcher has the game. Steam counts UpNext as the running game, so without that, Big Picture would keep waiting on UpNext after the game ends. Start UpNext from Steam again for the next pick. If the launcher couldn't be opened, or for a GOG game through Galaxy, UpNext stays open. Started any other way, UpNext stays open when a game starts.

## Your phone's home screen

Once a phone is paired, the browser's menu can add UpNext to the home screen: Chrome's "Add to home screen" or "Install app", Safari's Share › "Add to Home Screen". Over HTTPS ([Other devices](other-devices.md#https-and-proxies)) Chrome opens it in its own window. On an iPhone, pair again from inside the home-screen app: open it, then scan or type a new code.
