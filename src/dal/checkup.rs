use anyhow::Result;
use sqlx::Executor;

pub async fn insert(e: impl Executor<'_>, tick: i32) -> Result<()> {
  todo!()
}
