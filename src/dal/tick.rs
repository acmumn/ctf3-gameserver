use anyhow::Result;
use sqlx::{Connection, Executor};

use crate::models::Tick;

use super::Db;

pub async fn current(db: &Db) -> Result<Tick> {
  todo!()
}

pub async fn clear_in_progress(db: &Db, tick: i32) -> Result<()> {
  todo!()
}

pub async fn bump(db: &Db) -> Result<()> {
  // - Get tick number
  // - Update all flags to not be in progress
  // - Update the tick number

  let conn = db.acquire().await?;

  conn.transaction(|tx| {
    Box::pin(async {
      sqlx::query(
        "

      ",
      )
      .execute(db)
      .await?;

      Ok(())
    })
  })
}
