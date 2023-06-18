#[macro_use]
mod utils;

mod scoreboard;
mod submit_flag;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router, Server,
};
use warp::Filter;

use crate::config::Config;
use crate::db::Db;

use self::utils::set;

pub async fn run2(config: &Config) -> Result<()> {
    let app = Router::new()
        .route("/submit", post(submit_flag::submit_flag2))
        .route("/breakdown", post(scoreboard::breakdown_only2))
        .route("/check_up", post(scoreboard::check_up_only2))
        .route("/", get(scoreboard::scoreboard2));

    Server::bind(&config.bind_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[deprecated]
pub fn run_old(config: Config, bind_addr: SocketAddr, db: Db) {
    let ext = set(db).and(set(config));

    let routes = route_any!(
        POST("submit") => submit_flag::submit_flag(),
        GET("breakdown") => scoreboard::breakdown_only(),
        GET("check_up") => scoreboard::check_up_only(),
        GET() => scoreboard::scoreboard(),
    );

    warp::serve(ext.and(routes)).run(bind_addr)
}
