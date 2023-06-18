use std::{
  fmt,
  path::Path,
  process::{Output, Stdio},
  time::Duration,
};

use serde::{de, Serializer};
use tokio::{process::Command, time};

pub async fn child_output_helper<Args, Arg>(
  executable: impl AsRef<Path>,
  working_directory: impl AsRef<Path>,
  log_directory: impl AsRef<Path>,
  args: Args,
  timeout: Duration,
) -> Result<Output>
where
  Args: IntoIterator<Item = Arg>,
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

// https://github.com/ramsayleung/rspotify/issues/163#issuecomment-743719039

struct DurationVisitor;

impl<'de> de::Visitor<'de> for DurationVisitor {
  type Value = Duration;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "number of seconds")
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(Duration::from_secs(v))
  }
}

pub fn from_duration_sec<'de, D>(d: D) -> Result<Duration, D::Error>
where
  D: de::Deserializer<'de>,
{
  d.deserialize_u64(DurationVisitor)
}

pub fn to_duration_sec<S>(x: &Duration, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  s.serialize_u64(x.as_millis() as u64)
}
