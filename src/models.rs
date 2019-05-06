use chrono::NaiveDateTime;

use crate::schema::{check_ups, flags, services, teams};

#[derive(Queryable)]
pub struct Tick {
    pub id: i32,
    pub start_time: NaiveDateTime,
    pub current_tick: i32,
    pub current_check: i32,
}

#[derive(Queryable)]
pub struct Team {
    pub id: i32,
    pub arbitrary_bonus_points: i32,
    pub ip: i32,
}

#[derive(Insertable)]
#[table_name = "teams"]
pub struct NewTeam {
    pub id: i32,
    pub ip: i32,
}

#[derive(Debug, Queryable, Insertable)]
pub struct Service {
    pub name: String,
    pub port: i32,

    pub atk_score: i32,
    pub def_score: i32,
    pub up_score: i32,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Flag {
    pub tick: i32,
    pub team_id: i32,
    pub service_name: String,

    pub flag: String,
    pub flag_id: Option<String>,

    pub in_progress: bool,
    pub claimed_by: Option<i32>,
    pub defended: bool,
    pub created: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "flags"]
pub struct NewFlag {
    pub tick: i32,
    pub team_id: i32,
    pub service_name: String,

    pub flag: String,
    pub flag_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Serialize)]
pub struct CheckUp {
    pub id: i32,
    pub team_id: i32,
    pub service_name: String,
    pub in_progress: bool,
    pub up: bool,
    pub timestamp: NaiveDateTime,
}
