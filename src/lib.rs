pub extern crate tokio;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

pub mod check_up;
pub mod flag_io;
pub mod setup_logging;

mod config;
pub mod controllers;
pub mod dal;
mod game;
mod key;
pub mod models;
pub mod service;
pub mod utils;
pub mod web;

pub use crate::config::{Config, TeamConfig};
pub use crate::game::GameServer;
