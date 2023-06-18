use anyhow::{Context, Result};
use sqlx::Executor;

use crate::models::Flag;

pub async fn get_last_flag(e: impl Executor<'_>) -> Result<Flag> {
  sqlx::query(
    "
    ",
  )
  .execute(e)
  .await
  .context("could not execute query")?;
}
