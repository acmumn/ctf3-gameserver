use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use gameserver::{check_up, flag_io};
use gameserver::{Config, Db, GameServer};
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

    #[structopt(name = "migrate")]
    Migrate,
}

fn main() {
    env_logger::builder().default_format_timestamp(false).init();
    let opt = Opt::from_args();

    // read the config file
    let mut file = File::open(opt.config).expect("config file couldn't be opened");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("failed to read config");
    let config: Config = toml::from_slice(contents.as_slice()).expect("couldn't parse config");

    // connect to the db
    let db = Db::connect(&config.db).expect("couldn't connect to the db");

    match &opt.cmd {
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
        Command::Migrate => {
            db.migrate().expect("failed to migrate");
        }
    }
}
