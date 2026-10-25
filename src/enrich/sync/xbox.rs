//! The Xbox app's installed games, read from its install folders. Its
//! owned library, Game Pass included, has no sign-in UpNext may use: GOG
//! Galaxy or the games added by hand cover the rest. What is installed on
//! this computer goes to player 1's Xbox app connection, since this
//! computer's installs are theirs as the other machine defaults are.

use super::{Library, SyncRun, with_uninstalled};
use crate::config::Config;
use crate::db::SyncSummary;
use crate::enrich::{Clients, Progress};
use crate::xbox;
use anyhow::Result;
use async_trait::async_trait;

pub struct XboxApp;

fn found() -> Vec<xbox::XboxInstall> {
    let roots = xbox::roots();
    if roots.is_empty() {
        return vec![];
    }
    xbox::installed(&roots)
}

#[async_trait]
impl Library for XboxApp {
    fn id(&self) -> &'static str {
        "xbox"
    }
    fn label(&self) -> &'static str {
        "Xbox app"
    }
    fn blocked(&self, _cfg: &Config, _clients: &Clients) -> Option<&'static str> {
        found().is_empty().then_some("no Xbox app games installed here")
    }
    /// It also runs with nothing installed when it brought games in before,
    /// so the ones uninstalled since are marked so.
    fn ready(&self, run: &SyncRun<'_>) -> bool {
        self.blocked(run.cfg, run.clients).is_none()
            || run
                .db
                .sync_connections(xbox::SOURCE)
                .is_ok_and(|c| run.db.launcher_copies(c[0].owner, xbox::SOURCE, xbox::SOURCE).is_ok_and(|c| !c.is_empty()))
    }

    async fn sync(&self, run: &SyncRun<'_>, _progress: &mut Progress) -> Result<Vec<SyncSummary>> {
        let mut conn = run.db.sync_connections(xbox::SOURCE)?.remove(0);
        let whose = run.whose(&conn);
        let found = found();
        run.sink.log(format!("Xbox app{whose}: {} installed", found.len()));
        let games = with_uninstalled(run.db, conn.owner, xbox::SOURCE, xbox::SOURCE, found.iter().map(xbox::copy).collect())?;
        let s = run.db.sync_into(&mut conn, xbox::SOURCE, &games)?;
        run.sink.log(format!(
            "Xbox app{whose}: {} ({} new, {} updated, {} removed)",
            crate::enrich::plural(s.total, "game", "games"),
            s.added,
            s.updated,
            s.removed
        ));
        Ok(vec![s])
    }
}
