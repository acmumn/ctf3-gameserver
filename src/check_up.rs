use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use tokio::{prelude::*, timer::Delay};

use crate::GameServer;

pub fn ticker(gs: Arc<Mutex<GameServer>>) -> impl Future<Item = (), Error = ()> {
    let (interval, log_directory) = {
        let gs = gs.lock().unwrap();
        let config = gs.get_config();
        (config.check_period, config.log_directory.clone())
    };

    // get the latest tick number
    let check_number = {
        let gs = gs.lock().unwrap();
        let db = gs.get_db();
        db.get_current_check()
    }
    .unwrap();

    fn check_iter(
        check_number: i32,
        now: DateTime<Utc>,
        interval: u64,
        gs: Arc<Mutex<GameServer>>,
        log_directory: impl AsRef<Path>,
    ) -> impl Future<Item = (), Error = ()> + Send + Sync {
        // get teams
        let teams = {
            let gs = gs.clone();
            let gs = gs.lock().unwrap();
            gs.get_teams()
        };

        let mut fut: Box<Future<Item = (), Error = ()> + Send + Sync> = Box::new(future::ok(()));
        for team in teams {
            // log dir
            let log_dir = log_directory
                .as_ref()
                .join("check_up")
                .join(format!("team_{}", team.id));
            if !log_dir.exists() {
                fs::create_dir_all(&log_dir);
            }

            let gs = gs.clone();
            let gs = gs.lock().unwrap();
            let db = gs.get_db();
            fut = Box::new(
                fut.join(gs.check_up(db, check_number, now, team.id, team.ip, log_dir))
                    .map(|_| ()),
            );
        }

        fut = Box::new(fut.and_then(move |_| {
            let gs = gs.clone();
            let gs = gs.lock().unwrap();
            let db = gs.get_db();
            db.bump_checks().map_err(|err| {
                error!("Failed to bump tick: {}", err);
            })
        }));

        // delay
        Box::new(
            fut.join(
                Delay::new(Instant::now() + Duration::from_secs(interval)).map_err(|err| {
                    error!("Timer error: {}", err);
                }),
            )
            .map(|_| ()),
        )
    }

    stream::unfold(check_number, move |check_number| {
        let now = Utc::now();
        let interval = interval.into();
        Some(
            check_iter(
                check_number,
                now,
                interval,
                gs.clone(),
                log_directory.clone(),
            )
            .map(move |_| ((), check_number + 1)),
        )
    })
    .collect()
    .map(|_| ())
}
