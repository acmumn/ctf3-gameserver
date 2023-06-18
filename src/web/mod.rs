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
