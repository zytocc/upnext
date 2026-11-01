# Players, PINs and your data

## Players, PINs and hidden games

Several people can each have their own player ([Players](../players.md)). Players keep the household's picks, marks and taste apart; they don't keep secrets from each other.

- **No login.** Anyone at this computer, or at an allowed device that isn't bound to one player, can switch to any player who has no PIN. Anyone who can read UpNext's files can read every player's data, hidden games included.
- **PINs** are four to eight digits, kept only as an Argon2id hash. The picker asks for one before switching to its player, and so do their Hidden by me list, removing them, and binding a token or device to them. Five wrong tries lock that player's PIN for a minute, then two, four and so on. A PIN is a household courtesy, not security: anyone with the database file can guess it offline, and the command line on this computer doesn't ask for one.
- **Choosing a player.** A browser's choice is a signed `HttpOnly`, `SameSite=Strict` cookie (`Secure` over HTTPS), so nobody can choose a player with a PIN by writing it by hand. The `UpNext-Player` header names a player for a script, but not one with a PIN. While the default player has a PIN and nobody has been chosen, every per-player request answers `403`.
- **Bound devices and tokens.** A phone allowed in, paired or given a token for one player always acts for that player, whatever header or cookie it sends (`403` for another). It can't switch, add, rename or remove players, change another player's accounts or sharing, or change what is everyone's (the household settings, the schedules, setup). It can change its own player's settings and PIN.
- **Shared accounts.** A store or AI account's owner decides who it's offered to, and only the owner changes that, renames it or removes it. Its keys and sign-ins stay write-only for everyone, the owner included: a player using a shared AI account has its key used for them, not shown.
- **Hidden games** are hidden in UpNext only. The database, its backups and exports, and the command line on this computer hold them all. Job messages, their logs, the log download and `GET /api/logs` call a game hidden from a player "a hidden game" to that player; the command line's own log doesn't. A game hidden in the Steam client doesn't reach UpNext while `steam_include_hidden` is off.
- **Removing a player** is only on this computer, like a restore, and takes a backup first. It revokes the player's bound tokens and devices.

## Your data and keys

- The database and the backups are files in your user's data folder; `upnext doctor` prints the paths and [Running UpNext](../running.md) lists them.
- API keys you enter (Steam, RAWG, IGDB, Anthropic, OpenAI, an OpenAI-compatible server) are stored in that database in plain text, as are the GOG and Epic sign-ins. Anyone who can read your files can read them. On Linux and macOS UpNext makes the database and the backups readable by your user only (mode 600). On Windows they are under `%LOCALAPPDATA%`, which only your account and administrators can read.
- The API reports whether a key is set, not its value. The OpenAI-compatible server's key goes to the address set beside it, so changing that address from another device clears the saved key unless the same change brings a new one (and isn't allowed there when the key is in `.env`).
- A backup download or `upnext export` "without keys" leaves out:
  - keys, tokens and sign-ins;
  - the players' names (each becomes "Player N"), PINs and identity provider accounts;
  - the cookie's signing key;
  - store accounts' names and ids;
  - every setting that names a file, folder or address;
  - every job's log, result and error, and every AI call's error.

  Restored on the same computer, its own settings come back, but the players' identity provider links and the folders on other players' connections don't.

## Backlog verdicts

A [backlog verdict](../VERDICTS.md) is sent only when the player presses Send. [Consent and keys](../verdicts/consent.md) covers the key it carries.

Other players' verdicts are a daily download that sends nothing about you, and it isn't signed. `verdicts_url`, and an address given to `upnext verdicts apply`, must be `https://` (plain `http://` only to `localhost` or `127.0.0.1`). UpNext doesn't follow a redirect from `https://` to plain `http://`. [The published snapshot](../verdicts/snapshot.md) has the rest.

[Privacy](../privacy.md) lists what is sent to which service, the AI included.
