#[macro_use]
mod utils;

mod scoreboard;
mod submit_flag;

use std::net::SocketAddr;

use warp::Filter;

use crate::config::Config;
use crate::db::Db;

use self::utils::set;

pub fn run(config: Config, bind_addr: SocketAddr, db: Db) {
    let ext = set(db).and(set(config));

    let routes = route_any!(
        POST("submit") => submit_flag::submit_flag(),
        GET("breakdown") => scoreboard::breakdown_only(),
        GET("check_up") => scoreboard::check_up_only(),
        GET() => scoreboard::scoreboard(),
    );

    warp::serve(ext.and(routes)).run(bind_addr)
}
