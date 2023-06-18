use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use chrono::Duration;

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

    #[serde(
        default = "default_timeout_sec",
        serialize_with = "crate::utils::to_duration_sec",
        deserialize_with = "crate::utils::from_duration_sec"
    )]
    pub timeout_sec: u64,

    pub teams: Vec<TeamConfig>,

    pub db: PathBuf,
    pub services_dir: PathBuf,
    pub ignores: Vec<String>,

    pub log_directory: PathBuf,
    pub bind_addr: SocketAddr,
    pub secret_key: String,
}

fn default_timeout_sec() -> u64 {
    15
}
