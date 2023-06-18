use std::ffi::OsString;
use std::fs;
use std::io;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::time::{Duration, Instant};

use anyhow::Context;
use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::future;
use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;
use sqlx::SqlitePool;
use tokio::time::sleep;

use crate::dal;
use crate::key::generate_flag;
use crate::models::{self, Flag, NewFlag};
use crate::service::Service;
use crate::{Config, TeamConfig};

pub struct GameServer {
  db: SqlitePool,
  config: Config,
  services: Vec<Arc<Mutex<Service>>>,
  rng: StdRng,
}

impl GameServer {
  pub async fn new(config: Config, db: SqlitePool) -> Result<Self> {
    // create the log directory if it doesn't exist
    if !config.log_directory.exists() {
      fs::create_dir_all(&config.log_directory)
        .context("failed to create log directory")?;
    }

    // clear in-progress flags
    // let (current_tick, _) = db.get_current_tick().expect("failed to get current tick");
    let current_tick = dal::tick::current_tick(&db)
      .await
      .context("could not get current tick")?;
    dal::tick::clear_in_progress(&db, current_tick)
      .await
      .context("could not clear in-progress flags")?;

    // load teams into db
    for team in &config.teams {
      dal::team::create(&db, team.id, team.ip).await?;
      // db.add_team(team.id, team.ip).map_err(GameServerError::Db)?;
    }

    // list the directory
    let read_dir = fs::read_dir(&config.services_dir)
      .map_err(GameServerError::ListServices)?;
    let mut services = Vec::new();

    for entry in read_dir {
      let entry = match entry {
        Ok(entry) => entry,
        Err(err) => {
          services.push((String::new(), Err(GameServerError::ReadEntry(err))));
          continue;
        }
      };

      let name = match entry.file_name().into_string() {
        Ok(name) => name,
        Err(name) => {
          services.push((String::new(), Err(GameServerError::OsString(name))));
          continue;
        }
      };

      // ignore non-directories
      if !entry
        .file_type()
        .with_context(|| {
          format!(
            "could not stat service directory {}",
            entry.path().display()
          )
        })
        .is_dir()
      {
        continue;
      }

      // ignore directories starting with .
      if name.starts_with('.') {
        continue;
      }

      // ignore files listed explicitly in ignores
      if config.ignores.contains(&name) {
        continue;
      }

      let path = entry.path();
      let service = match Service::load_from_dir(&config, &name, &path)
        .map_err(GameServerError::Service)
        .and_then(|service| {
          db.add_service(&models::Service {
            name: name.clone(),
            port: service.config.port as i32,
            atk_score: service.config.atk_score as i32,
            def_score: service.config.def_score as i32,
            up_score: service.config.up_score as i32,
          })
          .map(|_| service)
          .map_err(GameServerError::Db)
        }) {
        Ok(service) => (name, Ok(service)),
        Err(err) => (name, Err(err)),
      };

      services.push(service);
    }

    let services = services
      .into_iter()
      .filter_map(|(name, service)| match service {
        Ok(service) => Some(Arc::new(Mutex::new(service))),
        Err(err) => {
          error!("Error loading {}: {:?}", name, err);
          None
        }
      })
      .collect();

    // use a pre-seeded RNG. not really "secure", but works for a game
    let rng = {
      let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
      StdRng::seed_from_u64(now)
    };

    let gameserver = GameServer {
      db,
      config,
      services,
      rng,
    };

    Ok(gameserver)
  }

  pub fn get_config(&self) -> &Config {
    &self.config
  }

  pub fn get_teams(&self) -> Vec<TeamConfig> {
    self.config.teams.clone()
  }

  pub fn get_db(&self) -> SqlitePool {
    self.db.clone()
  }

  pub async fn check_up(
    &self,
    check_number: i32,
    now: DateTime<Utc>,
    team_id: i32,
    target: Ipv4Addr,
    log_dir: impl AsRef<Path>,
  ) -> Result<()> {
    let services = self.services.clone();
    let delay = self.config.delay;
    let log_dir = log_dir.as_ref().to_path_buf();

    future::join_all(services.into_iter().map(move |service_mux| async {
      let service_mux2 = service_mux.clone();
      let service = service_mux.lock().unwrap();
      let name = service.name.clone();
      let log_dir = log_dir.clone();

      // choose a random delay and sleep
      let delay = self.rng.next_u64() % delay;
      let delay_duration = Duration::from_secs(delay);
      sleep(delay_duration).await;
      // let delay_timer = Delay::new(Instant::now() + Duration::from_secs(delay.into()));
      // let fut: Box<Future<Item = (), Error = GameServerError> + Send + Sync> =
      // Box::new(delay_timer.map(|_| ()).map_err(GameServerError::Delay));
      // let db = db.clone();

      let svc_name = name.clone();

      let insert_checkup = move |result: Result<_, _>| {
        db.insert_checkup(check_number, now, team_id, svc_name, result.is_ok())
          .map_err(GameServerError::Db)
      };

      let svc_name = name.clone();

      let service = service_mux2.lock().unwrap();
      info!("check_up service={} team_id={}", name, team_id);
      service
        .check_up(target, log_dir)
        .await
        .map_err(GameServerError::CheckUp)?;

      insert_checkup();
    }))
    .map(|_| ())
    .map_err(|err| {
      error!("gameserver check_up error: {:?}", err);
    });

    Ok(())
  }

  pub async fn each_team(
    &self,
    db: Db,
    tick: i32,
    team_id: i32,
    target: Ipv4Addr,
    has_prev: bool,
    get_log_dir: impl AsRef<Path>,
    set_log_dir: impl AsRef<Path>,
  ) -> Result<()> {
    let services = self.services.clone();
    let delay = self.config.delay;
    let get_log_dir = get_log_dir.as_ref().to_path_buf();
    let set_log_dir = set_log_dir.as_ref().to_path_buf();

    future::join_all(services.into_iter().map(move |service_mux| {
      let mut rng = rand::thread_rng();
      let db = db.clone();
      let service_name = {
        let service = service_mux.lock().unwrap();
        service.name.clone()
      };
      let set_log_dir = set_log_dir.join(&service_name);
      let get_log_dir = get_log_dir.join(&service_name);

      // choose a random delay
      let delay = rng.next_u32() % delay;
      let delay_timer =
        Delay::new(Instant::now() + Duration::from_secs(delay.into()));
      let mut fut2: Box<
        Future<Item = (), Error = GameServerError> + Send + Sync,
      > = Box::new(delay_timer.map(|_| ()).map_err(GameServerError::Delay));

      // there's no previous flag if the tick is 0
      if has_prev {
        let last_flag =
          future::result(db.get_last_flag(team_id, &service_name))
            .map_err(GameServerError::Db);
        let svc = service_mux.clone();
        let get_flag = move |last_flag: Flag| {
          let service = svc.lock().unwrap();
          let info = format!(
            "get_flag tick={} service={} team_id={} flag_id={:?}",
            tick, service.name, team_id, last_flag.flag_id
          );
          info!("{}", info);

          service
            .get_flag(target, last_flag.flag_id.clone(), get_log_dir)
            .map(move |result| {
              debug!(" {} => {:?}", info, result);
              (last_flag, result)
            })
            .map_err(GameServerError::GetFlag)
        };
        let db = db.clone();
        let db2 = db.clone();
        let service_name = service_name.clone();
        let svc_name = service_name.clone();
        fut2 = Box::new(
          fut2
            .and_then(|_| last_flag)
            .and_then(get_flag)
            .map(move |(last_flag, flag)| {
              (last_flag.tick, last_flag.flag == flag)
            })
            .and_then(move |(tick, result)| {
              db.update_defense(tick, team_id, service_name, result)
                .map_err(GameServerError::Db)
            })
            .or_else(move |err| {
              db2
                .update_defense(tick, team_id, svc_name, false)
                .map_err(GameServerError::Db)
                .and_then(|_| Err(err))
            }),
        );
      };

      // set the new flag
      let flag = generate_flag(tick, team_id, &service_name);
      let flag2 = flag.clone();

      let db = db.clone();
      let svc_name = service_name.clone();

      let service_name = service_name.clone();
      let svc_name2 = service_name.clone();
      let svc_mux = service_mux.clone();
      fut2
        .then(move |result| {
          let service = svc_mux.lock().unwrap();
          let info = format!(
            "set_flag tick={} service={} team_id={}",
            tick, service.name, team_id
          );
          info!("{}", info);

          let svc_name = service_name.clone();
          let insert_flag = move |flag_id| {
            debug!("  {} => {:?}", info, flag_id);
            let new_flag = NewFlag {
              flag: flag2.clone(),
              flag_id,
              team_id,
              tick,
              service_name: svc_name.clone(),
            };
            db.insert_flag(new_flag).map_err(GameServerError::Db)
          };

          let svc_name = service_name.clone();
          let set_flag = service
            .set_flag(target, flag, set_log_dir)
            .map_err(GameServerError::SetFlag);
          set_flag
            .and_then(insert_flag.clone())
            .or_else(move |err| insert_flag(None).and_then(|_| Err(err)))
            .and_then(|_| future::result(result))
            .or_else(move |err2| {
              warn!(
                "error with service={} team_id={}: {:?}",
                svc_name, team_id, err2
              );
              Ok(())
            })
        })
        .map(|_| ())
    }))
    .map(|_| ())
  }
}
