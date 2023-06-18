use anyhow::{Context, Result};
use sqlx::{Connection, Executor};

use crate::models::Flag;

pub async fn get_last_flag(db: impl Connection<'_>) -> Result<Flag> {
  sqlx::query(
    "
    ",
  )
  .execute(db)
  .await
  .context("could not execute query")?;
}

pub async fn find_by_flag(
  db: impl Connection<'_>,
  as_ref: &str,
) -> Result<Flag> {
  todo!()
}

pub async fn submit(db: impl Connection<'_>, team_id: TeamId) -> Result<Flag> {
  todo!()
}
