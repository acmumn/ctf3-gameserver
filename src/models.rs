use std::net::Ipv4Addr;

use chrono::{DateTime, Utc};

use crate::dal::team::TeamId;

// use crate::schema::{check_ups, flags, services, teams};

pub struct Tick {
  pub id: i32,
  pub start_time: DateTime<Utc>,
  pub current_tick: i32,
  pub current_check: i32,
}

pub struct Team {
  pub id: TeamId,
  pub arbitrary_bonus_points: i32,
  pub ip: Ipv4Addr,
}

pub struct NewTeam {
  pub id: i32,
  pub ip: i32,
}

#[derive(Debug)]
pub struct Service {
  pub name: String,
  pub port: i32,

  pub atk_score: i32,
  pub def_score: i32,
  pub up_score: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Flag {
  pub tick: i32,
  pub team_id: i32,
  pub service_name: String,

  pub flag: String,
  pub flag_id: Option<String>,

  pub in_progress: bool,
  pub claimed_by: Option<i32>,
  pub defended: bool,
  pub created: DateTime<Utc>,
}

impl Flag {
  #[inline]
  pub fn is_claimed(&self) -> bool {
    self.claimed_by.is_some()
  }
}

pub struct NewFlag {
  pub tick: i32,
  pub team_id: i32,
  pub service_name: String,

  pub flag: String,
  pub flag_id: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct CheckUp {
  pub id: i32,
  pub team_id: i32,
  pub service_name: String,
  pub in_progress: bool,
  pub up: bool,
  pub timestamp: DateTime<Utc>,
}
