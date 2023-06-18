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

pub async fn scoreboard2() {}

/*
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
*/

pub async fn check_up_only2() {}

// #[deprecated]
// pub fn check_up_only() -> Resp!() {
//   scoreboard_ctx()
//     .and_then(|mut ctx: Context| {
//       ctx.insert("show_left", &false);
//       TEMPLATE
//         .render("scoreboard.html", &ctx)
//         .map_err(|err| {
//           Error::Render(format!("{}, {:?}", err.to_string(), err.source()))
//         })
//         .map_err(reject)
//     })
//     .map(|body: String| {
//       Response::builder()
//         .header("content-type", "text/html")
//         .body(body)
//     })
//     .recover(|err| {
//       Ok(
//         Response::builder()
//           .header("content-type", "text/html")
//           .body(format!("Internal error: {:?}", err)),
//       )
//     })
//     .boxed()
// }

pub async fn breakdown_only2() {}

// #[deprecated]
// pub fn breakdown_only() -> Resp!() {
//   scoreboard_ctx()
//     .and_then(|mut ctx: Context| {
//       ctx.insert("show_right", &false);
//       TEMPLATE
//         .render("scoreboard.html", &ctx)
//         .map_err(|err| {
//           Error::Render(format!("{}, {:?}", err.to_string(), err.source()))
//         })
//         .map_err(reject)
//     })
//     .map(|body: String| {
//       Response::builder()
//         .header("content-type", "text/html")
//         .body(body)
//     })
//     .recover(|err| {
//       Ok(
//         Response::builder()
//           .header("content-type", "text/html")
//           .body(format!("Internal error: {:?}", err)),
//       )
//     })
//     .boxed()
// }
