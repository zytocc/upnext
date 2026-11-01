# How the service decides

The service in `services/verdicts/` ([its README](../../services/verdicts/README.md)) settles what [the contract](../VERDICTS.md) leaves open, and its tests hold it.

- **Addresses.** The client address is counted only as an HMAC under a random salt made for the day. The next daily run deletes the counters and the salt. Sends and reports count against the per-address limit. Withdrawing doesn't, and a blocked key can still withdraw.
- **Brigading.** A game's new verdicts are held once it gets more than 20 in a day. A game that had verdicts in the 28 days before is held at 5 or more in a day when that is more than ten times its daily average. Held verdicts count only after a moderator releases them, or after seven days with no reports.
- **Reported texts.** A text with reports from three different keys, each with a published verdict of its own, is held out of the next snapshot until a moderator looks. Its verdict's answers stay counted unless a moderator removes the whole verdict. Reports from keys with no published verdict don't hold a text, though the moderators see them.
- **Reports.** A report needs a verdict a snapshot has published. Replacing a verdict with the same text and stopping note keeps a held or hidden text as it was, with its reports. A changed text is shown, and the reports about the old one are cleared.
- **Withdrawn verdicts** are left out of the next snapshot. The daily run then deletes their answers and text and keeps only the id, the game, the key's hash and the days, so an id is never reused and a key that withdraws and sends again still counts against its limits.
- **One verdict a game.** One key has at most one live or held verdict a game.
- **Text checks.** A text or stopping note that fails today's text checks is left out of the snapshot, with its verdict's answers kept, so a stricter check can't make UpNext refuse the whole file.
