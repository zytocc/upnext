//! Box art from libretro's thumbnail server for games added by hand from an
//! emulator or a console (`crate::libretro`). Each system's list of names is
//! read once a month at most; a game is matched by its whole name.

use super::{Opt, Pass};
use crate::enrich::{Progress, Run};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

const OPTIONS: &[Opt] = &[Opt::new("libretro", "Box art from libretro")];

/// How long a system's list of names is kept before it's read again.
const KEEP_SECS: i64 = 30 * 24 * 3600;

pub struct Boxart;

#[async_trait]
impl Pass for Boxart {
    fn options(&self) -> &'static [Opt] {
        OPTIONS
    }
    fn name(&self) -> &'static str {
        "libretro box art"
    }
    fn wanted(&self, run: &Run<'_>) -> bool {
        run.on("libretro")
    }

    async fn run(&self, run: &Run<'_>) -> Result<()> {
        let pending: Vec<(i64, String, Vec<&'static str>)> = run
            .db
            .added_without_cover()?
            .into_iter()
            .map(|(id, name, from)| (id, name, crate::libretro::systems_for(&from)))
            .filter(|(_, _, systems)| !systems.is_empty())
            .collect();
        if pending.is_empty() {
            return Ok(());
        }
        let mut progress = Progress { total: pending.len(), done: 0, current: None, message: "Finding box art on libretro".into() };
        run.sink.update(&progress);
        let since = crate::db::now() - if run.refresh { 0 } else { KEEP_SECS };
        let mut lists: HashMap<&'static str, Vec<String>> = HashMap::new();
        let mut found = 0;
        for (copy, name, systems) in &pending {
            run.check_cancelled()?;
            for system in systems {
                if !lists.contains_key(system) {
                    let names = match run.db.libretro_boxarts(system, since)? {
                        Some(names) => names,
                        None => match run.clients.libretro.boxarts(system).await {
                            Ok(names) => {
                                run.db.put_libretro_boxarts(system, &names)?;
                                names
                            }
                            Err(e) => {
                                run.error(format!("libretro's {system} box art list failed: {e:#}"));
                                vec![]
                            }
                        },
                    };
                    lists.insert(system, names);
                }
                if let Some(art) = crate::libretro::best_match(name, &lists[system]) {
                    run.db.put_added_cover(*copy, &crate::libretro::boxart_url(system, art))?;
                    found += 1;
                    break;
                }
            }
            progress.done += 1;
            run.sink.update(&progress);
        }
        run.add("libretro", found);
        run.log(format!(
            "libretro: box art for {found} of {}",
            crate::enrich::plural(pending.len(), "game added by hand", "games added by hand")
        ));
        Ok(())
    }
}
