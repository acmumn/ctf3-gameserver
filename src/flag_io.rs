use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::future;
use tokio::time::sleep;

use crate::core::calculate_round_length;
use crate::{dal, game, GameServer};

// The main loop for inserting and retrieving flags
pub async fn main_loop(gameserver: GameServer) -> Result<()> {
  let (interval, log_directory) = {
    let gs = gameserver.lock().unwrap();
    let config = gs.get_config();
    (config.check_period, config.log_directory.clone())
  };

  // get the latest tick number
  let (tick_number, _) = {
    let gs = gameserver.lock().unwrap();
    let db = gs.get_db();
    db.get_current_tick()
  }
  .unwrap();

  let mut has_prev = false;

  loop {
    let round_length = calculate_round_length(tick_number, interval);

    info!(
      "=== TICK {} (has_prev={}): this round will last up to {}s",
      tick_number, has_prev, round_length
    );

    team_iter(
      has_prev,
      gameserver.clone(),
      round_length,
      tick_number,
      log_directory.to_path_buf(),
    )
    .await?;
    // .map(move |_| ((), (tick_number + 1, true))),

    has_prev = true;

    // Sleep for `round_length` seconds
    let duration = Duration::from_secs(round_length);
    sleep(duration).await?;
  }
}

async fn team_iter(
  has_prev: bool,
  gameserver: GameServer,
  round_length: u64,
  tick_number: i32,
  log_directory: impl AsRef<Path>,
) -> Result<()> {
  let log_directory = log_directory.as_ref();

  let db = gameserver.database();

  // Get a list of all teams
  let teams = dal::team::get_all(&db).await?;

  let futures = teams.into_iter().map(|team| async {
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

    gameserver
      .each_team(
        tick_number,
        team.id,
        team.ip,
        has_prev,
        get_log_dir,
        set_log_dir,
      )
      .await?;

    Ok(())
  });

  future::join_all(futures).await?;

  dal::tick::bump(&db);
}
