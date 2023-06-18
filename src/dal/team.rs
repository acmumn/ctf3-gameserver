use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use sqlx::Executor;

pub async fn create(
  db: impl Executor<'_>,
  team_id: i32,
  team_ip: Ipv4Addr,
) -> Result<()> {
  sqlx::query(
    "
    INSERT INTO teams (id, ip)
    VALUES (?, ?)",
  )
  .bind(team_id)
  .bind(team_ip)
  .execute(db)
  .await
  .context("could not insert team")?;
}
