//! Games added by hand from no store: one copy at a time in the player's
//! own account for the `other` source (`crate::added` checks what's typed).
//! Unlike the stores' lists (`crate::manual`), nothing replaces them: a
//! game stays until the player removes it.

use super::players::own_connection;
use super::sync::account_for;
use super::{Db, PlayerId, Refused, name_key, now};
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};

/// The source a game from no store has.
pub const OTHER: &str = "other";

/// What the player says about a game they add, already checked.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AddedGame {
    pub name: String,
    /// Where it's from, in their words: "RPCS3", "Meta Quest".
    pub from: Option<String>,
    pub cover_url: Option<String>,
    pub play_url: Option<String>,
    /// Hours played, as minutes; None when they didn't say.
    pub minutes: Option<i64>,
    /// When they last played it, Unix seconds; 0 when they didn't say.
    pub last_played: i64,
    pub installed: bool,
    /// Whether it's looked up by name on Steam, Wikidata and the rest.
    pub look_up: bool,
}

/// A game added by hand, as `GET /library/added` lists it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, utoipa::ToSchema)]
pub struct OwnGame {
    /// The copy's id (`games.id`), which the edit and remove routes take.
    pub copy_id: i64,
    /// The title it's grouped into, for `GET /games/{id}`.
    pub title_id: Option<i64>,
    pub name: String,
    pub from: Option<String>,
    pub cover_url: Option<String>,
    pub play_url: Option<String>,
    /// Hours played as minutes, or null when they weren't given.
    pub minutes: Option<i64>,
    pub last_played: i64,
    pub installed: bool,
    pub look_up: bool,
    /// Matched to a Steam app by name.
    pub steam_matched: bool,
}

/// The tables a title's look-ups by name filled. A game renamed or kept from
/// look-ups loses them, when no other copy shares its title.
const LOOKED_UP: [&str; 6] = ["steam_match", "xref", "igdb_meta", "rawg_meta", "pcgw_meta", "hltb"];

/// The covers the libretro pass sets, which a new name or place drops so
/// the game finds its own.
const LIBRETRO_ART: &str = "https://thumbnails.libretro.com/";

impl Db {
    /// Add a game from no store to `owner`'s library: a copy in their
    /// account for `other`, made with their `manual` connection when they
    /// have none. The copy's id and the title it joined.
    pub fn add_own_game(&self, owner: PlayerId, g: &AddedGame) -> Result<(i64, i64)> {
        self.with(|c| {
            let tx = c.unchecked_transaction()?;
            if same_game(&tx, owner, g, None)? {
                return Err(Refused(format!("{} is in your library already", describe(g))).into());
            }
            let connection = match own_connection(&tx, owner, crate::manual::LAUNCHER)? {
                Some(id) => id,
                None => {
                    tx.execute(
                        "INSERT INTO store_connections(kind, owner_id, created_at) VALUES (?1, ?2, ?3)",
                        params![crate::manual::LAUNCHER, owner, now()],
                    )?;
                    tx.last_insert_rowid()
                }
            };
            let account = account_for(&tx, connection, OTHER)?;
            let ts = now();
            tx.execute(
                "INSERT INTO games(source,source_id,launcher,name,installed,playtime_forever,playtime_known,rtime_last_played,
                    art_cover,run_url,l_from,look_up,first_seen_at,last_synced_at,removed,account_id)
                 VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13,0,?14)",
                params![
                    OTHER,
                    format!("added-{}", uuid::Uuid::new_v4().simple()),
                    crate::manual::LAUNCHER,
                    g.name,
                    g.installed as i64,
                    g.minutes.unwrap_or(0),
                    g.minutes.is_some() as i64,
                    g.last_played,
                    g.cover_url,
                    g.play_url,
                    g.from,
                    g.look_up as i64,
                    ts,
                    account
                ],
            )?;
            let id = tx.last_insert_rowid();
            Self::assign_titles_in(&tx)?;
            let title = tx.query_row("SELECT title_id FROM games WHERE id=?1", params![id], |r| r.get(0))?;
            tx.commit()?;
            Ok((id, title))
        })
    }

    /// Change a game `owner` added. A new name or a switch to keep it from
    /// look-ups drops its Steam match and, when it's the title's only copy,
    /// what the look-ups found, so a new name is looked up afresh. None when
    /// it isn't one of theirs.
    pub fn edit_own_game(&self, owner: PlayerId, copy_id: i64, g: &AddedGame) -> Result<Option<i64>> {
        self.with(|c| {
            let tx = c.unchecked_transaction()?;
            let Some(before) = own_row(&tx, owner, copy_id)? else { return Ok(None) };
            if same_game(&tx, owner, g, Some(copy_id))? {
                return Err(Refused(format!("{} is in your library already", describe(g))).into());
            }
            let renamed = name_key(&before.name) != name_key(&g.name);
            let moved = renamed || before.from != g.from;
            let kept_from = before.look_up && !g.look_up;
            if renamed || kept_from {
                if let Some(title) = before.title_id {
                    let alone: bool =
                        tx.query_row("SELECT COUNT(*) = 1 FROM games WHERE title_id=?1 AND removed=0", params![title], |r| r.get(0))?;
                    if alone {
                        for table in LOOKED_UP {
                            tx.execute(&format!("DELETE FROM {table} WHERE title_id=?1"), params![title])?;
                        }
                    }
                }
                tx.execute("UPDATE games SET appid=NULL, steam_matched=0 WHERE id=?1 AND steam_matched=1", params![copy_id])?;
            }
            // What the look-ups wrote on the copy itself is for the old name.
            let (genres, release) = if renamed || kept_from { (None, None) } else { (before.genres, before.release) };
            let cover = match (&g.cover_url, moved) {
                (Some(url), true) if url.starts_with(LIBRETRO_ART) => None,
                (url, _) => url.clone(),
            };
            tx.execute(
                "UPDATE games SET name=?2, l_from=?3, art_cover=?4, run_url=?5, installed=?6,
                    playtime_forever=?7, playtime_known=?8, rtime_last_played=?9, look_up=?10,
                    l_genres=?11, l_release=?12, last_synced_at=?13
                 WHERE id=?1",
                params![
                    copy_id,
                    g.name,
                    g.from,
                    cover,
                    g.play_url,
                    g.installed as i64,
                    g.minutes.unwrap_or(0),
                    g.minutes.is_some() as i64,
                    g.last_played,
                    g.look_up as i64,
                    genres,
                    release,
                    now()
                ],
            )?;
            Self::assign_titles_in(&tx)?;
            let title = tx.query_row("SELECT title_id FROM games WHERE id=?1", params![copy_id], |r| r.get(0))?;
            tx.commit()?;
            Ok(Some(title))
        })
    }

    /// Take a game `owner` added out of the library, as a sync takes out a
    /// game its store no longer reports: the title's status, ratings and
    /// diary stay with it. Whether it was one of theirs.
    pub fn remove_own_game(&self, owner: PlayerId, copy_id: i64) -> Result<bool> {
        self.with(|c| {
            let tx = c.unchecked_transaction()?;
            if own_row(&tx, owner, copy_id)?.is_none() {
                return Ok(false);
            }
            tx.execute("UPDATE games SET removed=1 WHERE id=?1", params![copy_id])?;
            Self::assign_titles_in(&tx)?;
            tx.commit()?;
            Ok(true)
        })
    }

    /// The games `owner` added from no store, newest first.
    pub fn own_games(&self, owner: PlayerId) -> Result<Vec<OwnGame>> {
        self.with(|c| {
            let mut st = c.prepare(&format!("{OWN_SELECT} ORDER BY g.first_seen_at DESC, g.id DESC"))?;
            let rows = st.query_map(params![owner, OTHER], own_game)?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }

    /// One game `owner` added, or None when it isn't one of theirs.
    pub fn own_game(&self, owner: PlayerId, copy_id: i64) -> Result<Option<OwnGame>> {
        self.with(|c| Ok(c.query_row(&format!("{OWN_SELECT} AND g.id=?3"), params![owner, OTHER, copy_id], own_game).optional()?))
    }

    /// The games added by hand that the libretro pass may find a cover for:
    /// no cover yet, looked up, a place they're from, and no Steam app (whose
    /// art Steam gives). (copy id, name, where it's from.)
    pub fn added_without_cover(&self) -> Result<Vec<(i64, String, String)>> {
        self.with(|c| {
            let mut st = c.prepare(
                "SELECT id, name, l_from FROM games
                 WHERE source=?1 AND removed=0 AND look_up=1 AND art_cover IS NULL AND appid IS NULL AND TRIM(COALESCE(l_from, '')) <> ''
                 ORDER BY id",
            )?;
            let rows = st.query_map(params![OTHER], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }

    /// Set the cover the libretro pass found for a game added by hand,
    /// unless the player set one meanwhile.
    pub fn put_added_cover(&self, copy_id: i64, url: &str) -> Result<()> {
        self.with(|c| {
            c.execute("UPDATE games SET art_cover=?2 WHERE id=?1 AND source=?3 AND art_cover IS NULL", params![copy_id, url, OTHER])?;
            Ok(())
        })
    }

    /// The games added by hand that Wikidata linked to an item, that no
    /// Steam app describes and that it wasn't asked about yet: (copy id,
    /// the item's QID).
    pub fn added_without_facts(&self) -> Result<Vec<(i64, String)>> {
        self.with(|c| {
            let mut st = c.prepare(
                "SELECT g.id, x.qid FROM games g JOIN xref x ON x.title_id = g.title_id
                 WHERE g.source=?1 AND g.removed=0 AND g.look_up=1 AND g.appid IS NULL AND x.success=1 AND x.qid IS NOT NULL
                   AND g.l_genres IS NULL
                 ORDER BY g.id",
            )?;
            let rows = st.query_map(params![OTHER], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }

    /// Record what Wikidata says of a game added by hand: its genres (an
    /// empty list when it names none, so it isn't asked again) and when it
    /// first came out ("2009-02-05"), each only where the copy has none.
    pub fn put_added_facts(&self, copy_id: i64, genres: &[String], released: Option<&str>) -> Result<()> {
        self.with(|c| {
            c.execute(
                "UPDATE games SET l_genres=COALESCE(l_genres, ?2), l_release=COALESCE(l_release, ?3) WHERE id=?1 AND source=?4",
                params![copy_id, super::to_json(&genres), released, OTHER],
            )?;
            Ok(())
        })
    }

    /// The box art names libretro has for `system`, when read since `since`.
    pub fn libretro_boxarts(&self, system: &str, since: i64) -> Result<Option<Vec<String>>> {
        self.with(|c| {
            let names: Option<String> = c
                .query_row("SELECT names FROM libretro_boxarts WHERE system=?1 AND fetched_at>=?2", params![system, since], |r| r.get(0))
                .optional()?;
            Ok(names.map(|n| super::json_vec(Some(n))))
        })
    }

    pub fn put_libretro_boxarts(&self, system: &str, names: &[String]) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT OR REPLACE INTO libretro_boxarts(system, fetched_at, names) VALUES (?1, ?2, ?3)",
                params![system, now(), super::to_json(&names)],
            )?;
            Ok(())
        })
    }
}

const OWN_SELECT: &str = "SELECT g.id, g.title_id, g.name, g.l_from, g.art_cover, g.run_url, g.playtime_forever, g.playtime_known,
        g.rtime_last_played, g.installed, g.look_up, g.steam_matched
     FROM games g JOIN store_accounts a ON a.id = g.account_id JOIN store_connections k ON k.id = a.connection_id
     WHERE k.owner_id=?1 AND g.source=?2 AND g.removed=0";

fn own_game(r: &rusqlite::Row) -> rusqlite::Result<OwnGame> {
    Ok(OwnGame {
        copy_id: r.get(0)?,
        title_id: r.get(1)?,
        name: r.get(2)?,
        from: r.get(3)?,
        cover_url: r.get(4)?,
        play_url: r.get(5)?,
        minutes: (r.get::<_, i64>(7)? != 0).then(|| r.get(6)).transpose()?,
        last_played: r.get(8)?,
        installed: r.get::<_, i64>(9)? != 0,
        look_up: r.get::<_, i64>(10)? != 0,
        steam_matched: r.get::<_, i64>(11)? != 0,
    })
}

/// The copy as it is before an edit.
struct Before {
    name: String,
    from: Option<String>,
    title_id: Option<i64>,
    look_up: bool,
    genres: Option<String>,
    release: Option<String>,
}

/// `copy_id`, when it's a game `owner` added from no store and still has.
fn own_row(c: &Connection, owner: PlayerId, copy_id: i64) -> Result<Option<Before>> {
    Ok(c.query_row(
        "SELECT g.name, g.l_from, g.title_id, g.look_up, g.l_genres, g.l_release
         FROM games g JOIN store_accounts a ON a.id = g.account_id JOIN store_connections k ON k.id = a.connection_id
         WHERE g.id=?1 AND k.owner_id=?2 AND g.source=?3 AND g.removed=0",
        params![copy_id, owner, OTHER],
        |r| {
            Ok(Before {
                name: r.get(0)?,
                from: r.get(1)?,
                title_id: r.get(2)?,
                look_up: r.get::<_, i64>(3)? != 0,
                genres: r.get(4)?,
                release: r.get(5)?,
            })
        },
    )
    .optional()?)
}

/// Whether `owner` already added a game of that name from the same place
/// (another than `except`).
fn same_game(c: &Connection, owner: PlayerId, g: &AddedGame, except: Option<i64>) -> Result<bool> {
    let mut st = c.prepare(
        "SELECT g.id, g.name, g.l_from FROM games g JOIN store_accounts a ON a.id = g.account_id
         JOIN store_connections k ON k.id = a.connection_id WHERE k.owner_id=?1 AND g.source=?2 AND g.removed=0",
    )?;
    let rows =
        st.query_map(params![owner, OTHER], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?)))?;
    let place = |f: &Option<String>| f.as_deref().map(name_key).unwrap_or_default();
    for row in rows {
        let (id, name, from) = row?;
        if Some(id) != except && name_key(&name) == name_key(&g.name) && place(&from) == place(&g.from) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// "Demon's Souls from RPCS3", or the name alone.
fn describe(g: &AddedGame) -> String {
    match &g.from {
        Some(f) => format!("{} from {f}", g.name),
        None => g.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("upnext.db"), 120).unwrap();
        (dir, db)
    }

    fn game(name: &str, from: Option<&str>) -> AddedGame {
        AddedGame { name: name.into(), from: from.map(str::to_string), look_up: true, ..Default::default() }
    }

    #[test]
    fn a_game_added_by_hand_joins_the_library_with_what_was_said() {
        let (_dir, db) = db();
        let g = AddedGame {
            minutes: Some(90),
            installed: true,
            play_url: Some("steam://rungameid/1".into()),
            ..game("Demon's Souls", Some("RPCS3"))
        };
        let (copy, title) = db.add_own_game(PlayerId::FIRST, &g).unwrap();
        let view = db.game(PlayerId::FIRST, title).unwrap().unwrap();
        let c = &view.copies[0];
        assert_eq!((c.game_id, c.source.as_str(), c.launcher.as_str(), c.from.as_deref()), (copy, "other", "manual", Some("RPCS3")));
        assert_eq!((c.installed, c.playtime_minutes, c.playtime_known, c.run_url.as_str()), (true, 90, true, "steam://rungameid/1"));
        assert!(view.looks_up());
        let listed = db.own_games(PlayerId::FIRST).unwrap();
        assert_eq!((listed.len(), listed[0].minutes, listed[0].title_id), (1, Some(90), Some(title)));
    }

    #[test]
    fn the_same_game_from_the_same_place_is_refused_but_another_place_is_not() {
        let (_dir, db) = db();
        db.add_own_game(PlayerId::FIRST, &game("Demon's Souls", Some("RPCS3"))).unwrap();
        let e = db.add_own_game(PlayerId::FIRST, &game("demon's souls", Some("rpcs3"))).unwrap_err();
        assert!(e.downcast_ref::<Refused>().is_some(), "{e:#}");
        db.add_own_game(PlayerId::FIRST, &game("Demon's Souls", Some("PS3 disc"))).unwrap();
        assert_eq!(db.own_games(PlayerId::FIRST).unwrap().len(), 2);
    }

    #[test]
    fn only_its_owner_edits_or_removes_it() {
        let (_dir, db) = db();
        let (copy, _) = db.add_own_game(PlayerId::FIRST, &game("Half-Life RTX", None)).unwrap();
        let sam = db.add_player("Sam").unwrap();
        assert_eq!(db.edit_own_game(sam, copy, &game("Mine now", None)).unwrap(), None);
        assert!(!db.remove_own_game(sam, copy).unwrap());
        assert!(db.own_games(sam).unwrap().is_empty());
        assert!(db.remove_own_game(PlayerId::FIRST, copy).unwrap());
        assert!(db.own_games(PlayerId::FIRST).unwrap().is_empty());
        assert!(db.all_games(PlayerId::FIRST).unwrap().is_empty());
    }

    #[test]
    fn a_rename_keeps_the_status_and_drops_the_old_names_look_ups() {
        let (_dir, db) = db();
        let (copy, title) = db.add_own_game(PlayerId::FIRST, &game("Persona 5", Some("RPCS3"))).unwrap();
        db.set_override(PlayerId::FIRST, title, Some(crate::db::Override::Shelved), None).unwrap();
        let title = db.put_steam_match(title, Some((1687950, "Persona 5 Royal", 0.9))).unwrap();
        db.put_added_cover(copy, "https://thumbnails.libretro.com/x.png").unwrap();
        let renamed = db.edit_own_game(PlayerId::FIRST, copy, &game("Persona 5 (PS3)", Some("RPCS3"))).unwrap().unwrap();
        let view = db.game(PlayerId::FIRST, renamed).unwrap().unwrap();
        assert_eq!((view.name.as_str(), view.appid, view.override_), ("Persona 5 (PS3)", None, Some(crate::db::Override::Shelved)));
        assert!(!view.has_match_attempt, "the new name is looked up afresh");
        assert_eq!(db.own_game(PlayerId::FIRST, copy).unwrap().unwrap().cover_url, None, "the old name's libretro cover goes");
        let _ = title;
    }

    #[test]
    fn kept_from_look_ups_it_loses_its_steam_match_and_is_not_looked_up() {
        let (_dir, db) = db();
        let (copy, title) = db.add_own_game(PlayerId::FIRST, &game("Skyrim", Some("a mod list"))).unwrap();
        db.put_steam_match(title, Some((72850, "The Elder Scrolls V: Skyrim", 0.9))).unwrap();
        let g = AddedGame { look_up: false, ..game("Skyrim", Some("a mod list")) };
        let title = db.edit_own_game(PlayerId::FIRST, copy, &g).unwrap().unwrap();
        let view = db.game(PlayerId::FIRST, title).unwrap().unwrap();
        assert_eq!((view.appid, view.steam_matched, view.looks_up()), (None, false, false));
    }

    #[test]
    fn a_stores_list_saved_again_leaves_games_from_no_store_alone() {
        let (_dir, db) = db();
        db.add_own_game(PlayerId::FIRST, &game("Gran Turismo 4", Some("PCSX2"))).unwrap();
        crate::manual::save(&db, PlayerId::FIRST, "origin", &["Dead Space".into()]).unwrap();
        crate::manual::save(&db, PlayerId::FIRST, "origin", &[]).unwrap();
        assert_eq!(db.own_games(PlayerId::FIRST).unwrap().len(), 1);
        assert!(crate::manual::lists(&db, PlayerId::FIRST).unwrap()["origin"].is_empty());
    }

    #[test]
    fn wikidata_facts_fill_only_what_the_copy_lacks() {
        let (_dir, db) = db();
        let (copy, title) = db.add_own_game(PlayerId::FIRST, &game("Okami", Some("PCSX2"))).unwrap();
        let x = crate::wikidata::Xref { qid: "Q1".into(), ..Default::default() };
        db.put_xref(title, Some(&x)).unwrap();
        assert_eq!(db.added_without_facts().unwrap(), [(copy, "Q1".to_string())]);
        db.put_added_facts(copy, &["action-adventure game".into()], Some("2006-04-20")).unwrap();
        assert!(db.added_without_facts().unwrap().is_empty());
        let view = db.game(PlayerId::FIRST, title).unwrap().unwrap();
        assert_eq!((view.genres.clone(), view.release_year()), (vec!["action-adventure game".to_string()], Some("2006".into())));
    }
}
