use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use sqlx::{Connection, Executor};

use crate::models::Team;

pub(crate) struct TeamId(pub(in crate::dal) u32);

pub async fn create(
  db: impl Executor<'_>,
  team_id: u32,
  team_ip: Ipv4Addr,
) -> Result<Team> {
  sqlx::query(
    "
    INSERT INTO teams (id, ip)
    VALUES (?, ?)
    ",
  )
  .bind(team_id)
  .bind(team_ip)
  .execute(db)
  .await
  .context("could not insert team")?;

  Team {
    id: TeamId(team_id),
    arbitrary_bonus_points: 0,
    ip: team_ip,
  }
}

pub async fn get_all(db: impl Connection<'_>) -> Result<Vec<Team>> {
  todo!()
}
