use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use sqlx::{Connection, Executor};

use crate::models::Team;

use super::Db;

pub(crate) struct TeamId(pub(in crate::dal) u32);

pub async fn create(db: Db, team_id: u32, team_ip: Ipv4Addr) -> Result<Team> {
  let team_ip_int = u32::from(team_ip);

  sqlx::query(
    "
    INSERT INTO teams (id, ip)
    VALUES (?, ?)
    ",
  )
  .bind(team_id)
  .bind(team_ip_int)
  .execute(db)
  .await
  .context("could not insert team")?;

  Ok(Team {
    id: TeamId(team_id),
    arbitrary_bonus_points: 0,
    ip: team_ip,
  })
}

pub async fn get_all(db: impl Connection) -> Result<Vec<Team>> {
  todo!()
}
