//! Games added by hand from no store: a game in an emulator, one played
//! only with a mod, a VR game from a headset's own store. What a player
//! types is checked here before the database keeps it (`db::added`); the
//! routes are in `server::stores` (docs/api/added-games.md).

use crate::db::{AddedGame, Refused};
use serde::Deserialize;

/// The longest name taken, as for the stores' lists.
const MAX_NAME_CHARS: usize = 200;

/// The longest "where it's from".
const MAX_FROM_CHARS: usize = 60;

/// The longest link taken.
const MAX_URL_CHARS: usize = 2_000;

/// The most hours a game can have been played, far past any real count.
const MAX_HOURS: f64 = 100_000.0;

/// The schemes a Play link may use: the launchers' own, which the desktop
/// app opens (`desktop::launcher_link`). Nothing that runs a program or
/// opens a file, since anyone who may change the library sets these.
pub const PLAY_SCHEMES: [&str; 5] = ["steam", "heroic", "goggalaxy", "com.epicgames.launcher", "uplay"];

/// A game as the Add a game form sends it.
#[derive(Debug, Clone, Default, Deserialize, utoipa::ToSchema)]
pub struct AddedBody {
    /// The game's name: "Demon's Souls".
    pub name: String,
    /// Where it's from, in the player's words: "RPCS3", "Meta Quest". An
    /// emulator or console's name also finds its box art on libretro.
    #[serde(default)]
    pub from: Option<String>,
    /// An `https` link to its cover.
    #[serde(default)]
    pub cover_url: Option<String>,
    /// A launcher link that starts it (`steam://rungameid/…`); see
    /// `PLAY_SCHEMES`.
    #[serde(default)]
    pub play_url: Option<String>,
    /// Hours played so far; left out when unknown.
    #[serde(default)]
    pub hours: Option<f64>,
    /// When it was last played: "2026-10-20".
    #[serde(default)]
    pub last_played: Option<String>,
    #[serde(default)]
    pub installed: bool,
    /// Look it up by name on Steam, Wikidata and the rest; on unless set false.
    #[serde(default = "yes")]
    pub look_up: bool,
}

fn yes() -> bool {
    true
}

/// Trimmed, on one line, without trademark signs; None when that leaves nothing.
fn tidy(s: Option<&str>) -> Option<String> {
    let s: String = s?.chars().filter(|c| !matches!(c, '™' | '®' | '©')).collect();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    (!s.is_empty()).then_some(s)
}

/// A link with one of `schemes`, or a refusal that says which.
fn link(s: Option<&str>, what: &str, schemes: &[&str], allowed: &str) -> Result<Option<String>, Refused> {
    let Some(s) = s.map(str::trim).filter(|s| !s.is_empty()) else { return Ok(None) };
    if s.chars().count() > MAX_URL_CHARS {
        return Err(Refused(format!("the {what} is too long; a link takes at most {MAX_URL_CHARS} characters")));
    }
    let scheme = reqwest::Url::parse(s).ok().map(|u| u.scheme().to_string());
    match scheme {
        Some(sc) if schemes.contains(&sc.as_str()) => Ok(Some(s.to_string())),
        _ => Err(Refused(format!("the {what} has to be {allowed}"))),
    }
}

impl AddedBody {
    /// What the database keeps, or why it won't. `today` is the date it is
    /// now, so a last-played date can't be in the future.
    pub fn checked(&self, today: chrono::NaiveDate) -> Result<AddedGame, Refused> {
        let name = tidy(Some(&self.name)).ok_or_else(|| Refused("give the game a name".into()))?;
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(Refused(format!("that name is too long; a name takes at most {MAX_NAME_CHARS} characters")));
        }
        let from = tidy(self.from.as_deref());
        if from.as_ref().is_some_and(|f| f.chars().count() > MAX_FROM_CHARS) {
            return Err(Refused(format!("where it's from takes at most {MAX_FROM_CHARS} characters")));
        }
        let cover_url = link(self.cover_url.as_deref(), "cover link", &["https"], "an https:// link")?;
        let play_url = link(self.play_url.as_deref(), "Play link", &PLAY_SCHEMES, "a launcher's link, such as steam://rungameid/…")?;
        let minutes = match self.hours {
            None => None,
            Some(h) if h.is_finite() && (0.0..=MAX_HOURS).contains(&h) => Some((h * 60.0).round() as i64),
            Some(_) => return Err(Refused(format!("hours played go from 0 to {}", MAX_HOURS as i64))),
        };
        let last_played = match self.last_played.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            None => 0,
            Some(d) => {
                let date = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|_| Refused(format!("{d:?} isn't a date; give the last time played as 2026-10-20")))?;
                if date > today {
                    return Err(Refused("the last time played can't be in the future".into()));
                }
                date.and_hms_opt(12, 0, 0).map(|t| t.and_utc().timestamp()).unwrap_or(0)
            }
        };
        Ok(AddedGame { name, from, cover_url, play_url, minutes, last_played, installed: self.installed, look_up: self.look_up })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 10, 24).unwrap()
    }

    fn body(name: &str) -> AddedBody {
        AddedBody { name: name.into(), look_up: true, ..Default::default() }
    }

    #[test]
    fn what_is_typed_is_tidied() {
        let b = AddedBody {
            from: Some("  RPCS3 ".into()),
            hours: Some(1.5),
            last_played: Some("2026-10-20".into()),
            cover_url: Some(" ".into()),
            ..body("  Demon's   Souls™ ")
        };
        let g = b.checked(today()).unwrap();
        assert_eq!((g.name.as_str(), g.from.as_deref(), g.minutes, g.cover_url), ("Demon's Souls", Some("RPCS3"), Some(90), None));
        assert_eq!(g.last_played, 1_792_497_600, "noon UTC on the day");
        assert!(g.look_up);
    }

    #[test]
    fn a_play_link_must_be_a_launchers() {
        let with = |url: &str| AddedBody { play_url: Some(url.into()), ..body("Game") }.checked(today());
        assert_eq!(with("steam://rungameid/12345").unwrap().play_url.as_deref(), Some("steam://rungameid/12345"));
        assert!(with("heroic://launch/gog/1").is_ok());
        for bad in ["file:///usr/bin/rpcs3", "javascript:alert(1)", "https://example.org/play", "rpcs3 --no-gui game.iso"] {
            assert!(with(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn covers_are_https_links() {
        let with = |url: &str| AddedBody { cover_url: Some(url.into()), ..body("Game") }.checked(today());
        assert!(with("https://example.org/cover.png").is_ok());
        assert!(with("http://example.org/cover.png").is_err());
        assert!(with(&format!("https://example.org/{}", "a".repeat(MAX_URL_CHARS))).is_err());
    }

    #[test]
    fn nonsense_is_refused_with_a_reason() {
        let e = |b: AddedBody| b.checked(today()).unwrap_err().0;
        assert_eq!(e(body("  ")), "give the game a name");
        assert!(e(body(&"x".repeat(201))).contains("200"));
        assert!(e(AddedBody { from: Some("y".repeat(61)), ..body("Game") }).contains("60"));
        assert!(e(AddedBody { hours: Some(-1.0), ..body("Game") }).contains("hours"));
        assert!(e(AddedBody { hours: Some(f64::NAN), ..body("Game") }).contains("hours"));
        assert!(e(AddedBody { last_played: Some("2026-10-25".into()), ..body("Game") }).contains("future"));
        assert!(e(AddedBody { last_played: Some("last week".into()), ..body("Game") }).contains("isn't a date"));
    }

    #[test]
    fn look_up_is_on_unless_turned_off() {
        let b: AddedBody = serde_json::from_str(r#"{"name": "Okami"}"#).unwrap();
        assert!(b.look_up && !b.installed);
        let b: AddedBody = serde_json::from_str(r#"{"name": "Okami", "look_up": false}"#).unwrap();
        assert!(!b.look_up);
    }
}
