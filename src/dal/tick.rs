use anyhow::Result;
use sqlx::{Connection, Executor};

pub async fn current_tick(e: impl Executor<'_>) -> Result<i32> {
  todo!()
}

pub async fn clear_in_progress(e: impl Executor<'_>, tick: i32) -> Result<()> {
  todo!()
}

pub async fn bump(db: impl Connection) -> Result<()> {
  // - Get tick number
  // - Update all flags to not be in progress
  // - Update the tick number

  Ok(())
}
