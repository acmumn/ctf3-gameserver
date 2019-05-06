use std::error::Error as StdError;
use std::net::Ipv4Addr;
use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error::{self as DieselError, NotFound, RollbackTransaction};
use diesel_migrations::RunMigrationsError;

use crate::models::{CheckUp, Flag, NewFlag, NewTeam, Service, Team, Tick};

embed_migrations!("migrations");

macro_rules! match_flag {
    ($tick:expr, $team_id:expr, $service_name:expr) => {{
        use crate::schema::flags::dsl;
        dsl::tick
            .eq($tick)
            .and(dsl::team_id.eq($team_id))
            .and(dsl::service_name.eq($service_name))
    }};
}

#[derive(Clone)]
pub struct Db(Arc<Pool<ConnectionManager<PgConnection>>>);

struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

#[derive(Debug, Display)]
pub enum DbError {
    Pool(r2d2::Error),
    GetConn(r2d2::Error),
    Migration(RunMigrationsError),
    Diesel(DieselError),
    InsertTeam(DieselError),
    GetLastFlag(DieselError),
    GetAllFlags(DieselError),
    GetAllTeams(DieselError),
    GetAllServices(DieselError),
    GetAllCheckup(DieselError),
    InsertFlag(DieselError),
    InsertService(DieselError),
    InsertCheckup(DieselError),
    UpdateDefense(DieselError),
    LookupFlag(DieselError),
    UpdateClaim(DieselError),
    Tick(DieselError),
}

impl StdError for DbError {}

impl Db {
    pub fn connect(database_url: impl AsRef<str>) -> Result<Self, DbError> {
        let database_url = database_url.as_ref();
        let manager = ConnectionManager::new(database_url);
        let pool = Pool::new(manager).map_err(DbError::Pool)?;
        Ok(Db(Arc::new(pool)))
    }

    fn get_conn(&self) -> Result<DbConn, DbError> {
        self.0.get().map(DbConn).map_err(DbError::GetConn)
    }

    pub fn migrate(&self) -> Result<(), DbError> {
        let conn = self.get_conn()?;
        embedded_migrations::run(&conn.0).map_err(DbError::Migration)
    }

    pub fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce() -> Result<R, DbError>,
    {
        let conn = self.get_conn()?;
        let mut err = None;
        let result = conn.0.transaction(|| match f() {
            Ok(v) => Ok(v),
            Err(e) => {
                err = Some(e);
                Err(RollbackTransaction)
            }
        });
        match result {
            Ok(v) => Ok(v),
            Err(_) => Err(err.unwrap()),
        }
    }

    pub fn clear_in_progress(&self, tick: i32) -> Result<(), DbError> {
        let conn = self.get_conn()?;
        self.transaction(|| {
            {
                use crate::schema::flags::dsl::{flags, in_progress, tick};
                diesel::delete(flags.filter(in_progress.eq(true)))
                    .execute(&conn.0)
                    .map_err(DbError::Diesel)?;
            }
            {
                use crate::schema::check_ups::dsl::{check_ups, in_progress};
                diesel::delete(check_ups.filter(in_progress.eq(true)))
                    .execute(&conn.0)
                    .map_err(DbError::Diesel)?;
            }
            Ok(())
        })
    }

    pub fn add_team(&self, team_id: i32, team_ip: Ipv4Addr) -> Result<(), DbError> {
        use crate::schema::teams::dsl::{id, teams};
        let conn = self.get_conn()?;
        self.transaction(|| {
            let team = match teams.filter(id.eq(team_id)).first::<Team>(&conn.0) {
                Ok(v) => Some(v),
                Err(NotFound) => None,
                Err(err) => return Err(DbError::InsertTeam(err)),
            };

            if team.is_none() {
                // insert team
                let new_team = NewTeam {
                    id: team_id,
                    ip: u32::from(team_ip) as i32,
                };

                use crate::schema::teams;
                diesel::insert_into(teams::table)
                    .values(&new_team)
                    .execute(&conn.0)
                    .map_err(DbError::InsertTeam)?;
            }

            Ok(())
        })
    }

    pub fn add_service(&self, new_service: &Service) -> Result<(), DbError> {
        use crate::schema::services::dsl::{name, services};
        let conn = self.get_conn()?;
        self.transaction(|| {
            let service = match services
                .filter(name.eq(&new_service.name))
                .first::<Service>(&conn.0)
            {
                Ok(v) => Some(v),
                Err(NotFound) => None,
                Err(err) => return Err(DbError::InsertService(err)),
            };

            if service.is_none() {
                use crate::schema::services;
                diesel::insert_into(services::table)
                    .values(new_service)
                    .execute(&conn.0)
                    .map_err(DbError::InsertService)?;
            }

            Ok(())
        })
    }

    pub fn get_current_tick(&self) -> Result<(i32, NaiveDateTime), DbError> {
        use crate::schema::tick::dsl::tick;
        let conn = self.get_conn()?;
        tick.first::<Tick>(&conn.0)
            .map(|row| (row.current_tick, row.start_time))
            .map_err(DbError::Tick)
    }

    pub fn get_current_check(&self) -> Result<i32, DbError> {
        use crate::schema::tick::dsl::tick;
        let conn = self.get_conn()?;
        tick.first::<Tick>(&conn.0)
            .map(|row| row.current_check)
            .map_err(DbError::Tick)
    }

    pub fn bump_tick(&self) -> Result<(), DbError> {
        use crate::schema::flags::dsl::{flags, in_progress, tick as flag_tick};
        use crate::schema::tick::dsl::{current_tick, start_time, tick};
        let conn = self.get_conn()?;
        self.transaction(|| {
            // get the current tick number
            let tick_number = tick
                .first::<Tick>(&conn.0)
                .map(|row| row.current_tick)
                .map_err(DbError::Diesel)?;

            // update all flags to not be in progress
            diesel::update(flags.filter(flag_tick.eq(tick_number)))
                .set(in_progress.eq(false))
                .execute(&conn.0)
                .map(|_| ())
                .map_err(DbError::Tick)?;

            // update the tick number
            diesel::update(tick)
                .set((
                    current_tick.eq(tick_number + 1),
                    start_time.eq(Utc::now().naive_utc()),
                ))
                .execute(&conn.0)
                .map(|_| ())
                .map_err(DbError::Tick)?;

            Ok(())
        })
    }

    pub fn bump_checks(&self) -> Result<(), DbError> {
        use crate::schema::check_ups::dsl::{check_ups, in_progress};
        use crate::schema::tick::dsl::{current_check, start_time, tick};
        let conn = self.get_conn()?;
        self.transaction(|| {
            // get the current tick number
            let check_number = tick
                .first::<Tick>(&conn.0)
                .map(|row| row.current_check)
                .map_err(DbError::Diesel)?;

            diesel::update(check_ups)
                .set(in_progress.eq(false))
                .execute(&conn.0)
                .map(|_| ())
                .map_err(DbError::Tick)?;

            // update the tick number
            diesel::update(tick)
                .set(current_check.eq(check_number + 1))
                .execute(&conn.0)
                .map(|_| ())
                .map_err(DbError::Tick)?;

            Ok(())
        })
    }

    pub fn get_all_teams(&self) -> Result<Vec<Team>, DbError> {
        use crate::schema::teams::dsl::teams;
        let conn = self.get_conn()?;
        teams.load(&conn.0).map_err(DbError::GetAllTeams)
    }

    pub fn get_all_flags(&self) -> Result<Vec<Flag>, DbError> {
        use crate::schema::flags::dsl::flags;
        let conn = self.get_conn()?;
        flags.load(&conn.0).map_err(DbError::GetAllFlags)
    }

    pub fn get_all_services(&self) -> Result<Vec<Service>, DbError> {
        use crate::schema::services::dsl::services;
        let conn = self.get_conn()?;
        services.load(&conn.0).map_err(DbError::GetAllServices)
    }

    pub fn get_all_checkups(&self) -> Result<Vec<CheckUp>, DbError> {
        use crate::schema::check_ups::dsl::{check_ups, timestamp};
        let conn = self.get_conn()?;
        check_ups
            .order(timestamp.desc())
            .load(&conn.0)
            .map_err(DbError::GetAllCheckup)
    }

    pub fn insert_checkup(
        &self,
        check_number: i32,
        now: DateTime<Utc>,
        team_id: i32,
        service_name: impl AsRef<str>,
        up: bool,
    ) -> Result<(), DbError> {
        use crate::schema::check_ups;
        let service_name = service_name.as_ref();
        let conn = self.get_conn()?;
        let new_checkup = CheckUp {
            id: check_number,
            timestamp: now.naive_utc(),
            team_id,
            service_name: service_name.to_owned(),
            in_progress: true,
            up,
        };
        diesel::insert_into(check_ups::table)
            .values(&new_checkup)
            .execute(&conn.0)
            .map(|_| ())
            .map_err(DbError::InsertCheckup)
    }

    pub fn get_last_flag(
        &self,
        team_id: i32,
        service_name: impl AsRef<str>,
    ) -> Result<Flag, DbError> {
        use crate::schema::flags::dsl::{self, flags};
        let service_name = service_name.as_ref();
        let conn = self.get_conn()?;
        flags
            .filter(
                dsl::team_id
                    .eq(team_id)
                    .and(dsl::service_name.eq(service_name)),
            )
            .order(dsl::tick.desc())
            .first(&conn.0)
            .map_err(DbError::GetLastFlag)
    }

    pub fn insert_flag(&self, new_flag: NewFlag) -> Result<(), DbError> {
        use crate::schema::flags;
        let conn = self.get_conn()?;
        diesel::insert_into(flags::table)
            .values(&new_flag)
            .execute(&conn.0)
            .map(|_| ())
            .map_err(DbError::InsertFlag)
    }

    pub fn lookup_flag(&self, flag: impl AsRef<str>) -> Result<Flag, DbError> {
        use crate::schema::flags::dsl::{self, flags};
        let flag = flag.as_ref();
        let conn = self.get_conn()?;
        flags
            .filter(dsl::flag.eq(flag))
            .first(&conn.0)
            .map_err(DbError::LookupFlag)
    }

    pub fn update_defense(
        &self,
        tick: i32,
        team_id: i32,
        service_name: impl AsRef<str>,
        result: bool,
    ) -> Result<(), DbError> {
        use crate::schema::flags::dsl::{defended, flags, in_progress};
        let service_name = service_name.as_ref();
        let conn = self.get_conn()?;
        diesel::update(flags.filter(match_flag!(tick, team_id, service_name)))
            .set((defended.eq(result), in_progress.eq(false)))
            .execute(&conn.0)
            .map_err(DbError::UpdateDefense)
            .map(|_| ())
    }

    pub fn claim_flag(&self, flag: &Flag, claimed_by: i32) -> Result<(), DbError> {
        use crate::schema::flags::dsl::{self, flags};
        let conn = self.get_conn()?;
        diesel::update(flags.filter(match_flag!(flag.tick, flag.team_id, &flag.service_name)))
            .set(dsl::claimed_by.eq(claimed_by))
            .execute(&conn.0)
            .map_err(DbError::UpdateClaim)
            .map(|_| ())
    }
}
