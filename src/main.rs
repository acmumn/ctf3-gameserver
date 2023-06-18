use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use gameserver::{check_up, flag_io, setup_logging};
use gameserver::{Config, Db, GameServer};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use structopt::StructOpt;
use tokio::prelude::*;

#[derive(StructOpt)]
struct Opt {
    #[structopt(flatten)]
    cmd: Command,

    #[structopt(long = "config")]
    config: PathBuf,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "run")]
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();

    setup_logging::setup_logging();

    // read the config file
    let mut file = File::open(opts.config).context("config file couldn't be opened")?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .context("failed to read config")?;
    let config: Config = toml::from_slice(contents.as_slice()).context("couldn't parse config")?;

    // connect to the db
    let db_options = SqliteConnectOptions::new().filename(&config.db);
    let db = SqlitePool::connect_with(db_options).await?;
    // let db = Db::connect(&config.db).expect("couldn't connect to the db");

    match opts.cmd {
        Command::Run => {
            let bind_addr = config.bind_addr;

            let gameserver = GameServer::new(config.clone()).expect("couldn't load gameserver");
            let gameserver = Arc::new(Mutex::new(gameserver));

            let check_up = check_up::ticker(gameserver.clone());
            let flag_io = flag_io::ticker(gameserver.clone());

            thread::spawn(move || {
                gameserver::web::run(config, bind_addr, db);
            });
            tokio::run(check_up.join(flag_io).map(|_| ()));
        }
    }

    Ok(())
}
