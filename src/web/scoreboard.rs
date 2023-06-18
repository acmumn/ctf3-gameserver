use std::collections::HashMap;
use std::error::Error as StdError;

use chrono::{Duration, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use tera::{Context, Tera};
use warp::{http::Response, reject::custom as reject, Filter, Rejection};

use crate::config::Config;
use crate::core::calculate_round_length;
use crate::models::{CheckUp, Flag};

lazy_static! {
  static ref TEMPLATE: Tera = {
    let mut tera = Tera::default();
    tera
      .add_raw_template("util.html", include_str!("util.html"))
      .unwrap();
    tera
      .add_raw_template("scoreboard.html", include_str!("scoreboard.html"))
      .unwrap();
    tera
  };
}

#[derive(Default, Serialize)]
struct SummaryEntry {
  pub id: i32,
  pub atk_score: u32,
  pub def_score: u32,
  pub up_score: u32,
  pub total_score: u32,
}

#[derive(Clone, Default, Serialize)]
struct TickEntry {
  pub number: i32,
  pub in_progress: bool,
  pub data: HashMap<i32, HashMap<String, Flag>>,
}

#[derive(Serialize)]
struct UptimeEntry {
  pub number: i32,
  pub in_progress: bool,
  pub data: HashMap<i32, HashMap<String, CheckUp>>,
}

#[derive(Debug, Display)]
enum Error {
  Db(DbError),

  MissingService(String),
  MissingTeam(i32),

  Render(String),
}

impl StdError for Error {}

fn scoreboard_ctx() -> impl Filter<Extract = (Context,), Error = Rejection> {
  warp::ext::get::<Config>()
    .and(warp::ext::get::<Db>())
    .and_then(|config: Config, db: Db| -> Result<Context, Rejection> {
      // TODO: convert these all to SQL queries...
      let mut ctx = Context::new();
      ctx.insert("show_left", &true);
      ctx.insert("show_right", &true);

      let (tick_number, start_time) =
        db.get_current_tick().map_err(Error::Db).map_err(reject)?;
      let round_length =
        calculate_round_length(tick_number, config.flag_period);
      let now = Utc::now().naive_utc();
      let remaining_time = start_time
        + Duration::seconds(round_length as i64 + config.delay as i64)
        - now;
      let round_start_ago = &(now - start_time);
      ctx.insert("round_start_ago", &round_start_ago.num_seconds());
      ctx.insert("round_length", &(round_length + config.delay as u64));
      ctx.insert("remaining_time", &remaining_time.num_seconds());

      let services = db
        .get_all_services()
        .map_err(Error::Db)
        .map_err(reject)?
        .into_iter()
        .map(|service| (service.name.clone(), service))
        .collect::<HashMap<_, _>>();
      ctx.insert(
        "services",
        &services.iter().map(|(name, _)| name).collect::<Vec<_>>(),
      );

      let mut teams = db
        .get_all_teams()
        .map_err(Error::Db)
        .map_err(reject)?
        .into_iter()
        .map(|team| {
          (
            team.id,
            SummaryEntry {
              id: team.id,
              ..Default::default()
            },
          )
        })
        .collect::<HashMap<_, _>>();

      let flags = db.get_all_flags().map_err(Error::Db).map_err(reject)?;
      let mut ticks = HashMap::new();
      for flag in flags {
        // skip current tick
        if flag.tick == tick_number {
          continue;
        }

        ticks.entry(flag.tick).or_insert(TickEntry {
          number: flag.tick,
          in_progress: flag.in_progress,
          data: HashMap::new(),
        });
        let mut this_tick = ticks.get_mut(&flag.tick).unwrap();
        this_tick.number = flag.tick;

        if flag.in_progress {
          this_tick.in_progress = true;
          continue;
        }

        this_tick
          .data
          .entry(flag.team_id)
          .or_insert_with(HashMap::new);
        let this_tick_team = this_tick.data.get_mut(&flag.team_id).unwrap();
        this_tick_team.insert(flag.service_name.clone(), flag.clone());

        let service = services
          .get(&flag.service_name)
          .ok_or_else(|| Error::MissingService(flag.service_name.clone()))
          .map_err(reject)?;
        if let Some(team_id) = flag.claimed_by {
          let mut team = teams
            .get_mut(&team_id)
            .ok_or_else(|| Error::MissingTeam(team_id))
            .map_err(reject)?;
          team.atk_score += service.atk_score as u32;
        } else if flag.defended {
          let mut team = teams
            .get_mut(&flag.team_id)
            .ok_or_else(|| Error::MissingTeam(flag.team_id))
            .map_err(reject)?;
          team.def_score += service.def_score as u32;
        }
      }

      let check_ups =
        db.get_all_checkups().map_err(Error::Db).map_err(reject)?;
      let mut checks = HashMap::new();
      for check_up in check_ups {
        checks.entry(check_up.id).or_insert(UptimeEntry {
          number: check_up.id,
          in_progress: false,
          data: HashMap::new(),
        });
        let mut this_check = checks.get_mut(&check_up.id).unwrap();

        if check_up.in_progress {
          this_check.in_progress = true;
          continue;
        }

        this_check
          .data
          .entry(check_up.team_id)
          .or_insert_with(HashMap::new);
        let this_check_team =
          this_check.data.get_mut(&check_up.team_id).unwrap();
        this_check_team.insert(check_up.service_name.clone(), check_up.clone());

        let service = services
          .get(&check_up.service_name)
          .ok_or_else(|| Error::MissingService(check_up.service_name.clone()))
          .map_err(reject)?;

        if check_up.up {
          let mut team = teams
            .get_mut(&check_up.team_id)
            .ok_or_else(|| Error::MissingTeam(check_up.team_id))
            .map_err(reject)?;
          team.up_score += service.up_score as u32;
        }
      }

      ctx.insert(
        "teams",
        &teams
          .values_mut()
          .map(|team| {
            team.total_score = team.atk_score + team.def_score + team.up_score;
            team
          })
          .collect::<Vec<_>>(),
      );
      ctx.insert("ticks", &{
        let mut v = ticks
          .into_iter()
          .filter_map(
            |(_, item)| {
              if !item.in_progress {
                Some(item)
              } else {
                None
              }
            },
          )
          .collect::<Vec<_>>();
        v.sort_unstable_by_key(|item| -item.number);
        v
      });
      ctx.insert("checks", &{
        let mut v = checks
          .into_iter()
          .filter_map(
            |(_, item)| {
              if !item.in_progress {
                Some(item)
              } else {
                None
              }
            },
          )
          .collect::<Vec<_>>();
        v.sort_unstable_by_key(|item| item.number);
        v
      });
      Ok(ctx)
    })
    .boxed()
}

pub async fn scoreboard2() {}

#[deprecated]
pub fn scoreboard() -> Resp!() {
  scoreboard_ctx()
    .and_then(|ctx| {
      TEMPLATE
        .render("scoreboard.html", &ctx)
        .map_err(|err| {
          Error::Render(format!("{}, {:?}", err.to_string(), err.source()))
        })
        .map_err(reject)
    })
    .map(|body: String| {
      Response::builder()
        .header("content-type", "text/html")
        .body(body)
    })
    .recover(|err| {
      Ok(
        Response::builder()
          .header("content-type", "text/html")
          .body(format!("Internal error: {:?}", err)),
      )
    })
    .boxed()
}

pub async fn check_up_only2() {}

#[deprecated]
pub fn check_up_only() -> Resp!() {
  scoreboard_ctx()
    .and_then(|mut ctx: Context| {
      ctx.insert("show_left", &false);
      TEMPLATE
        .render("scoreboard.html", &ctx)
        .map_err(|err| {
          Error::Render(format!("{}, {:?}", err.to_string(), err.source()))
        })
        .map_err(reject)
    })
    .map(|body: String| {
      Response::builder()
        .header("content-type", "text/html")
        .body(body)
    })
    .recover(|err| {
      Ok(
        Response::builder()
          .header("content-type", "text/html")
          .body(format!("Internal error: {:?}", err)),
      )
    })
    .boxed()
}

pub async fn breakdown_only2() {}

#[deprecated]
pub fn breakdown_only() -> Resp!() {
  scoreboard_ctx()
    .and_then(|mut ctx: Context| {
      ctx.insert("show_right", &false);
      TEMPLATE
        .render("scoreboard.html", &ctx)
        .map_err(|err| {
          Error::Render(format!("{}, {:?}", err.to_string(), err.source()))
        })
        .map_err(reject)
    })
    .map(|body: String| {
      Response::builder()
        .header("content-type", "text/html")
        .body(body)
    })
    .recover(|err| {
      Ok(
        Response::builder()
          .header("content-type", "text/html")
          .body(format!("Internal error: {:?}", err)),
      )
    })
    .boxed()
}
