use std::fs::{self, File};
use std::io::{self, Read};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::time;

use crate::config::Config;

/// A service runs on the competitors' machine and sends periodic updates to the
/// gameserver.
pub struct Service {
  pub name: String,
  pub config: ServiceConfig,

  pub timeout: Duration,
  pub base_dir: PathBuf,

  pub get_flag_path: PathBuf,
  pub set_flag_path: PathBuf,
  pub check_up_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceConfig {
  pub port: u32,

  pub atk_score: u32,
  pub def_score: u32,
  pub up_score: u32,

  pub get_flag_path: Option<PathBuf>,
  pub set_flag_path: Option<PathBuf>,
  pub check_up_path: Option<PathBuf>,
}

macro_rules! optional_path {
  (let $var_name:ident = ($opt_1:expr , $opt_2:expr)) => {
    let $var_name = match $opt_1 {
      Some(var) => var.to_owned(),
      None => $opt_2,
    };

    if !$var_name.exists() {
      bail!("could not locate file: {}", $var_name.display());
    }
  };
}

impl Service {
  pub fn load_from_dir(
    config: &Config,
    name: impl AsRef<str>,
    path: impl AsRef<Path>,
  ) -> Result<Self> {
    let name = name.as_ref();
    let path = path.as_ref();

    let config_path = path.join("meta.toml");
    if !config_path.exists() {
      bail!("configuration file is missing");
    }

    let mut config_file =
      File::open(&config_path).context("could not open config file")?;
    let mut contents = String::new();
    config_file
      .read_to_string(&mut contents)
      .context("could not read config file")?;
    let config: ServiceConfig =
      toml::from_str(&contents).context("could not parse config file")?;

    optional_path!(let get_flag_path = (&config.get_flag_path, path.join("get_flag")));
    optional_path!(let set_flag_path = (&config.set_flag_path, path.join("set_flag")));
    optional_path!(let check_up_path = (&config.check_up_path, path.join("check_up")));

    let service = Service {
      name: name.to_owned(),
      config,
      timeout: config.timeout,
      base_dir: path.to_path_buf(),
      get_flag_path,
      set_flag_path,
      check_up_path,
    };
    Ok(service)
  }

  pub async fn get_flag(
    &self,
    target: Ipv4Addr,
    flag_id: Option<String>,
    log_dir: impl AsRef<Path>,
  ) -> Result<String> {
    let executable = self.get_flag_path.to_owned();
    let port = self.config.port;

    let mut args = vec![target.to_string(), port.to_string()];
    if let Some(flag_id) = flag_id {
      args.push(flag_id);
    }

    let child_output =
      child_output_helper(executable, &self.base_dir, &self.timeout).await?;

    let string = String::from_utf8(child_output.stdout)?;
    let string = string.trim().to_owned();

    Ok(string)
  }

  /// Check if the service is still up (heartbeat)
  pub async fn check_up(
    &self,
    target: Ipv4Addr,
    log_dir: impl AsRef<Path>,
  ) -> Result<()> {
    let name = self.name.clone();
    let executable = self.check_up_path.to_owned();
    let port = self.config.port;

    let args = vec![target.to_string(), port.to_string()];
    let child = TimeoutCommand::new(
      executable,
      &self.base_dir,
      log_dir.as_ref(),
      args,
      Duration::from_secs(self.timeout as u64),
    )
    .map_err(ServiceError::Spawn);

    future::result(child)
      .and_then(|child| child.map_err(ServiceError::Subprocess))
      .map(|_| ())
  }

  pub fn set_flag(
    &self,
    target: Ipv4Addr,
    flag: impl AsRef<str>,
    log_dir: impl AsRef<Path>,
  ) -> impl Future<Item = Option<String>, Error = ServiceError> {
    let flag = flag.as_ref().to_owned();
    let executable = self.set_flag_path.to_owned();
    let port = self.config.port;

    let args = vec![target.to_string(), port.to_string(), flag];
    let child = TimeoutCommand::new(
      executable.clone(),
      &self.base_dir,
      log_dir.as_ref(),
      args.clone(),
      Duration::from_secs(self.timeout as u64),
    )
    .map_err(ServiceError::Spawn);

    future::result(child)
      .and_then(|child| child.map_err(ServiceError::Subprocess))
      .and_then(|output| {
        String::from_utf8(output)
          .map(|output| {
            let output = output.trim().to_owned();
            if !output.is_empty() {
              Some(output)
            } else {
              None
            }
          })
          .map_err(ServiceError::DecodeOutput)
      })
  }
}

async fn child_output_helper(
  executable: impl AsRef<Path>,
  working_directory: impl AsRef<Path>,
  log_directory: impl AsRef<Path>,
  timeout: Duration,
) -> Result<Output> {
  let child_future = Command::new(executable.as_ref())
    .current_dir(working_directory.as_ref())
    .stdin(Stdio::null())
    .stderr(Stdio::inherit())
    .stdout(Stdio::piped())
    .spawn()
    .context("could not spawn child")?;

  let child_output = time::timeout(timeout, child_future.wait_with_output())
    .await
    .context("child execution failed")??;

  Ok(child_output)
}
