use std::{fmt, net::Ipv4Addr};

use anyhow::{Context, Result};
use sqlx::{Connection, Executor, Row};

use crate::models::Team;

use super::Db;

#[derive(Debug)]
pub(crate) struct TeamId(pub(in crate::dal) u32);

impl fmt::Display for TeamId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<Team {}>", self.0)
  }
}

pub async fn create(db: &Db, team_id: u32, team_ip: Ipv4Addr) -> Result<Team> {
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

pub async fn get_all(db: &Db) -> Result<Vec<Team>> {
  Ok(
    sqlx::query("SELECT * FROM teams")
      .fetch_all(db)
      .await
      .context("could not fetch all teams")?
      .into_iter()
      .map(|row| Team {
        id: TeamId(row.get::<u32, _>("id")),
        arbitrary_bonus_points: row.get("arbitrary_bonus_points"),
        ip: Ipv4Addr::from(row.get::<u32, _>("ip")),
      })
      .collect(),
  )
}
