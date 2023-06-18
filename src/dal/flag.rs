use anyhow::{Context, Result};
use sqlx::{Connection, Executor};

use crate::models::Flag;

use super::team::TeamId;

pub async fn get_last_flag(db: impl Connection) -> Result<Flag> {
  sqlx::query(
    "
    ",
  )
  .execute(db)
  .await
  .context("could not execute query")?;
}

pub async fn find_by_flag(db: impl Connection, as_ref: &str) -> Result<Flag> {
  todo!()
}

pub async fn submit(db: impl Connection, team_id: TeamId) -> Result<Flag> {
  todo!()
}
