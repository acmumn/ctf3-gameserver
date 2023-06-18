use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::time::sleep;

use crate::{dal, game, GameServer};

/// The main loop for checking competitors' server health status
pub async fn main_loop(gameserver: GameServer) -> Result<()> {
  let (interval, log_directory) = {
    let gs = gameserver.lock().unwrap();
    let config = gs.get_config();
    (config.check_period, config.log_directory.clone())
  };

  // get the latest tick number
  let mut check_number = {
    let gs = gameserver.lock().unwrap();
    let db = gs.get_db();
    db.get_current_check()
  }
  .unwrap();

  loop {
    let now = Utc::now();

    check_iter_helper(
      check_number,
      now,
      interval,
      gameserver.clone(),
      log_directory.clone(),
    );

    // Sleep for `interval` seconds
    let duration = Duration::from_secs(interval);
    sleep(duration).await?;
  }
}

async fn check_iter_helper(
  check_number: i32,
  now: DateTime<Utc>,
  interval: u64,
  gameserver: GameServer,
  log_directory: impl AsRef<Path>,
) -> Result<()> {
  // Get a list of all teams
  let teams = dal::team::get_all(gameserver.database()).await?;

  for team in teams {
    // log dir
    let log_dir = log_directory
      .as_ref()
      .join("check_up")
      .join(format!("team_{}", team.id));
    if !log_dir.exists() {
      fs::create_dir_all(&log_dir);
    }

    let gs = gameserver.clone();
    let gs = gs.lock().unwrap();
    let db = gs.get_db();

    fut = Box::new(
      fut
        .join(gs.check_up(db, check_number, now, team.id, team.ip, log_dir))
        .map(|_| ()),
    );
  }

  fut = Box::new(fut.and_then(move |_| {
    let gs = gameserver.clone();
    let gs = gs.lock().unwrap();
    let db = gs.get_db();
    db.bump_checks().map_err(|err| {
      error!("Failed to bump tick: {}", err);
    })
  }));
}
