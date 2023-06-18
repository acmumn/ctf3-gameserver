use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use futures::future;
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
    sleep(duration).await;
  }
}

async fn check_iter_helper(
  check_number: i32,
  now: DateTime<Utc>,
  interval: u64,
  gameserver: GameServer,
  log_directory: impl AsRef<Path>,
) -> Result<()> {
  let db = gameserver.database();

  // Get a list of all teams
  let teams = dal::team::get_all(&db).await?;

  let log_directory = log_directory.as_ref().to_path_buf();
  let futures = teams.into_iter().map(move |team| {
    let log_directory = log_directory.clone();
    let gameserver = gameserver.clone();

    async move {
      // log dir
      let log_dir = log_directory.join("check_up").join(team.id.to_string());

      if !log_dir.exists() {
        fs::create_dir_all(&log_dir);
      }

      gameserver
        .check_up(check_number, now, team.id, team.ip, log_dir)
        .await?;

      Ok::<_, Error>(())
    }
  });

  future::join_all(futures).await;

  dal::checkup::bump(&db).await?;

  Ok(())
}
