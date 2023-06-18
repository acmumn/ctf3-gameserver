//! Database abstraction layer

use std::ops::Deref;

use axum::async_trait;
use futures::{future::BoxFuture, stream::BoxStream};
use sqlx::{
  database::HasStatement, Database, Describe, Either, Error, Execute, Executor,
  Sqlite, SqlitePool,
};

pub mod checkup;
pub mod flag;
pub mod team;
pub mod tick;

#[derive(Debug, Clone)]
pub struct Db(pub SqlitePool);

impl Deref for Db {
  type Target = SqlitePool;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'c> Executor<'c> for &Db {
  type Database = Sqlite;

  fn fetch_many<'e, 'q: 'e, E: 'q>(
    self,
    query: E,
  ) -> BoxStream<
    'e,
    Result<
      Either<
        <Self::Database as Database>::QueryResult,
        <Self::Database as Database>::Row,
      >,
      Error,
    >,
  >
  where
    'c: 'e,
    E: Execute<'q, Self::Database>,
  {
    self.0.fetch_many(query)
  }

  fn fetch_optional<'e, 'q: 'e, E: 'q>(
    self,
    query: E,
  ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>, Error>>
  where
    'c: 'e,
    E: Execute<'q, Self::Database>,
  {
    self.0.fetch_optional(query)
  }

  fn prepare_with<'e, 'q: 'e>(
    self,
    sql: &'q str,
    parameters: &'e [<Self::Database as Database>::TypeInfo],
  ) -> BoxFuture<
    'e,
    Result<<Self::Database as HasStatement<'q>>::Statement, Error>,
  >
  where
    'c: 'e,
  {
    self.0.prepare_with(sql, parameters)
  }

  fn describe<'e, 'q: 'e>(
    self,
    sql: &'q str,
  ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
  where
    'c: 'e,
  {
    self.0.describe(sql)
  }
}
