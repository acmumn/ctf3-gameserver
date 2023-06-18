use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamConfig {
    pub id: i32,
    pub ip: Ipv4Addr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub flag_period: u32,
    pub check_period: u32,
    pub delay: u64,
    pub timeout: u32,
    pub teams: Vec<TeamConfig>,

    pub db: PathBuf,
    pub services_dir: PathBuf,
    pub ignores: Vec<String>,

    pub log_directory: PathBuf,
    pub bind_addr: SocketAddr,
    pub secret_key: String,
}
