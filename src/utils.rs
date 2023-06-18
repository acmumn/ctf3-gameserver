use std::ops::Deref;
use std::{
  ffi::OsStr,
  fmt,
  path::Path,
  process::{Output, Stdio},
  time::Duration,
};

use anyhow::{Context, Result};
use serde::{de, Deserialize, Serialize, Serializer};
use tokio::{process::Command, time};

/// Run a child process with a timeout and the specified common args
pub async fn child_output_helper<Args, Arg>(
  executable: impl AsRef<Path>,
  working_directory: impl AsRef<Path>,
  log_directory: impl AsRef<Path>,
  args: Args,
  timeout: Duration,
) -> Result<Output>
where
  Args: IntoIterator<Item = Arg>,
  Arg: AsRef<OsStr>,
{
  let child_future = Command::new(executable.as_ref())
    .args(args)
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

/// Integer number of seconds that can be decoded from serde
#[derive(Copy, Clone, Debug)]
pub struct Seconds(pub Duration);

impl Deref for Seconds {
  type Target = Duration;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Into<Duration> for Seconds {
  fn into(self) -> Duration {
    self.0
  }
}

impl Serialize for Seconds {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let seconds = self.0.as_secs();
    seconds.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Seconds {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let seconds = u64::deserialize(deserializer)?;
    let duration = Duration::from_secs(seconds);
    Ok(Seconds(duration))
  }
}
