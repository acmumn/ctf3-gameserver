use anyhow::Error;
use axum::{
  async_trait,
  extract::{FromRequest, State},
  http::Request,
};
use chrono::Duration;
use sqlx::{Sqlite, SqlitePool};

use crate::dal::Db;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoreboardContext {
  /// How long ago the round started (in seconds)
  pub round_start_ago: u32,
}

#[async_trait]
impl<B> FromRequest<Db, B> for ScoreboardContext
where
  B: Send,
{
  type Rejection = Error;

  async fn from_request(
    req: Request<B>,
    state: &Db,
  ) -> Result<Self, Self::Rejection> {
    let db = State::<Db>::from_request(req, state).await?;

    todo!()
  }
}
