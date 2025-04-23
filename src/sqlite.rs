#![allow(dead_code)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use sqlx::{Database, Describe, Either, Execute, Executor, Result, SqlitePool};

#[derive(Clone, Debug)]
pub struct SqliteDb {
    pub pool: SqlitePool,
}

impl SqliteDb {
    pub async fn new() -> Self {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        Self { pool }
    }
}

/// A delegate to the underlying `sqlx::SqlitePool`.
///
/// This allows us to use the `SqliteDb` as an executor for SQL queries.
impl<'c> Executor<'c> for &SqliteDb {
    type Database = sqlx::Sqlite;

    fn fetch_many<'e, 'q: 'e, E: 'q + Execute<'q, Self::Database>>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        Result<
            Either<<Self::Database as Database>::QueryResult, <Self::Database as Database>::Row>,
        >,
    > {
        self.pool.fetch_many(query)
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q + Execute<'q, Self::Database>>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>>> {
        self.pool.fetch_optional(query)
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<'e, Result<<Self::Database as Database>::Statement<'q>>> {
        self.pool.prepare_with(sql, parameters)
    }

    fn describe<'e, 'q: 'e>(self, sql: &'q str) -> BoxFuture<'e, Result<Describe<Self::Database>>> {
        self.pool.describe(sql)
    }
}
