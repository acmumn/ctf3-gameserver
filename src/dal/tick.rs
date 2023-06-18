use sqlx::Executor;

pub async fn current_tick(e: impl Executor<'_>) -> Result<i32> {
  todo!()
}

pub async fn clear_in_progress(e: impl Executor<'_>, tick: i32) -> Result<()> {
  todo!()
}
