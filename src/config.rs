use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use crate::utils::Seconds;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
  pub flag_period: Seconds,
  pub check_period: Seconds,
  pub delay: Seconds,
  pub timeout: Seconds,

  pub teams: Vec<TeamConfig>,

  pub db: PathBuf,
  pub services_dir: PathBuf,
  pub ignores: Vec<String>,

  pub log_directory: PathBuf,
  pub bind_addr: SocketAddr,
  pub secret_key: String,
}

fn default_timeout_sec() -> Duration {
  Duration::from_secs(15)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamConfig {
  pub id: u32,
  pub ip: Ipv4Addr,
}
