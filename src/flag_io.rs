use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::{prelude::*, timer::Delay};

use crate::GameServer;

pub fn calculate_round_length(tick_number: i32, interval_sec: u32) -> u64 {
    let tick = f64::from(tick_number);
    let interval = f64::from(interval_sec) / 60.0;
    let delay = 2.0 * interval * (-2.0 * tick / interval).exp() + interval;
    (delay * 60.0) as u64
}

pub fn ticker(gs: Arc<Mutex<GameServer>>) -> impl Future<Item = (), Error = ()> {
    let (interval, log_directory) = {
        let gs = gs.lock().unwrap();
        let config = gs.get_config();
        (config.check_period, config.log_directory.clone())
    };

    // get the latest tick number
    let (tick_number, _) = {
        let gs = gs.lock().unwrap();
        let db = gs.get_db();
        db.get_current_tick()
    }
    .unwrap();

    fn team_iter(
        has_prev: bool,
        gs: Arc<Mutex<GameServer>>,
        round_length: u64,
        tick_number: i32,
        log_directory: impl AsRef<Path>,
    ) -> impl Future<Item = (), Error = ()> + Send + Sync {
        let log_directory = log_directory.as_ref();

        // get teams
        let teams = {
            let gs = gs.lock().unwrap();
            gs.get_teams()
        };

        let mut fut: Box<Future<Item = (), Error = ()> + Send + Sync> = Box::new(future::ok(()));
        for team in teams {
            // log dir
            let log_directory = log_directory.to_path_buf();
            let team_str = format!("team_{:02}", team.id);
            let tick_str = format!("tick_{:03}", tick_number);
            let get_log_dir = log_directory
                .join("get_flag")
                .join(&team_str)
                .join(&tick_str);
            if !get_log_dir.exists() {
                fs::create_dir_all(&get_log_dir);
            }
            let set_log_dir = log_directory
                .join("set_flag")
                .join(&team_str)
                .join(&tick_str);
            if !set_log_dir.exists() {
                fs::create_dir_all(&set_log_dir);
            }

            let gs = gs.lock().unwrap();
            let db = gs.get_db();
            fut = Box::new(
                fut.join(gs.each_team(
                    db,
                    tick_number,
                    team.id,
                    team.ip,
                    has_prev,
                    get_log_dir,
                    set_log_dir,
                ))
                .map(|_| ()),
            );
        }

        fut = Box::new(fut.and_then(move |_| {
            let gs = gs.clone();
            let gs = gs.lock().unwrap();
            let db = gs.get_db();
            db.bump_tick().map_err(|err| {
                error!("Failed to bump tick: {}", err);
            })
        }));

        // delay
        Box::new(
            fut.join(
                Delay::new(Instant::now() + Duration::from_secs(round_length)).map_err(|err| {
                    error!("Timer error: {}", err);
                }),
            )
            .map(|_| ()),
        )
    }

    stream::unfold((tick_number, false), move |(tick_number, has_prev)| {
        let round_length = calculate_round_length(tick_number, interval);
        info!(
            "=== TICK {} (has_prev={}): this round will last up to {}s",
            tick_number, has_prev, round_length
        );
        Some(
            team_iter(
                has_prev,
                gs.clone(),
                round_length,
                tick_number,
                log_directory.to_path_buf(),
            )
            .map(move |_| ((), (tick_number + 1, true))),
        )
    })
    .collect()
    .map(|_| ())
}
