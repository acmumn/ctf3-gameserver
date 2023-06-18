use std::fs::{self, File};
use std::io::{self, Read};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

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

#[derive(Debug)]
pub enum ServiceError {
    ConfigFileMissing,
    OpenConfigFile(io::Error),
    ReadConfigFile(io::Error),
    ParseConfig(toml::de::Error),
    FileNotFound(PathBuf),
    DecodeOutput(std::string::FromUtf8Error),
    Spawn(io::Error),
    Subprocess(TimeoutCommandError),

    GetFlagIO(io::Error),
    SetFlagIO(io::Error),
    CheckUpIO(io::Error),

    GetFlagError,
    SetFlagError,
    CheckUpError,
}

macro_rules! optional_path {
    (let $var_name:ident = ($opt_1:expr , $opt_2:expr)) => {
        let $var_name = match $opt_1 {
            Some(var) => var.to_owned(),
            None => $opt_2,
        };

        if !$var_name.exists() {
            return Err(ServiceError::FileNotFound($var_name));
        }
    };
}

impl Service {
    pub fn load_from_dir(
        gs_config: &Config,
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
    ) -> Result<Self, ServiceError> {
        let name = name.as_ref();
        let path = path.as_ref();

        let config_path = path.join("meta.toml");
        if !config_path.exists() {
            return Err(ServiceError::ConfigFileMissing);
        }

        let mut config_file = File::open(&config_path).map_err(ServiceError::OpenConfigFile)?;
        let mut contents = String::new();
        config_file
            .read_to_string(&mut contents)
            .map_err(ServiceError::ReadConfigFile)?;
        let config: ServiceConfig = toml::from_str(&contents).map_err(ServiceError::ParseConfig)?;

        optional_path!(let get_flag_path = (&config.get_flag_path, path.join("get_flag")));
        optional_path!(let set_flag_path = (&config.set_flag_path, path.join("set_flag")));
        optional_path!(let check_up_path = (&config.check_up_path, path.join("check_up")));

        let service = Service {
            name: name.to_owned(),
            config,
            timeout: gs_config.timeout,
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
    ) -> Result<String, Error = ServiceError> {
        let executable = self.get_flag_path.to_owned();
        let port = self.config.port;

        let mut args = vec![target.to_string(), port.to_string()];
        if let Some(flag_id) = flag_id {
            args.push(flag_id);
        }

        let child_future = Command::new(executable)
            .current_dir(&self.base_dir)
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .spawn_async();
        let child = timeout(self.timeout, child_future).spawn()?;

        let output = child.wait_with_output().await?;

        future::result(child)
            .and_then(|child| child.map_err(ServiceError::Subprocess))
            .and_then(|output| {
                String::from_utf8(output)
                    .map(|output| output.trim().to_owned())
                    .map_err(ServiceError::DecodeOutput)
            })
    }

    /// Check if the service is still up (heartbeat)
    pub async fn check_up(
        &self,
        target: Ipv4Addr,
        log_dir: impl AsRef<Path>,
    ) -> Result<(), ServiceError> {
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
