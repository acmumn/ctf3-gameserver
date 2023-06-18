use anyhow::Result;
use sqlx::{Connection, Executor};

use crate::dal::{self, Db};

pub async fn submit(db: &Db, flag: impl AsRef<str>) -> Result<()> {
  (*db).transaction(|| async {
    // Look up the flag
    let flag = dal::flag::find_by_flag(db, flag.as_ref()).await?;

    // if the flag's already been claimed, fail
    if flag.claimed_by.is_some() {
      bail!("flag already claimed")
    }

    // Wait can i do this all in the DAL lmao
    todo!()
  });

  Ok(())
}
