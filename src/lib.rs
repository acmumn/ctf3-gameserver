pub extern crate tokio;

#[macro_use]
extern crate derive_more;
#[macro_use]
pub extern crate diesel;
#[macro_use]
pub extern crate diesel_migrations;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod check_up;
pub mod flag_io;

mod config;
pub mod db;
mod game;
mod key;
pub mod models;
pub mod schema;
pub mod service;
pub mod util;
pub mod web;

pub use crate::config::{Config, TeamConfig};
pub use crate::db::{Db, DbError};
pub use crate::game::GameServer;
