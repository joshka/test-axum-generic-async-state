use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::FromRow;

use crate::AppState;
use crate::memory_db::MemoryDb;
use crate::sqlite::SqliteDb;

pub fn routes() -> Router<AppState> {
    Router::new().route("/{id}", get(user::<Arc<dyn UserRepository>>))
}

async fn user<R: UserRepository>(
    id: Path<u32>,
    repository: State<R>,
) -> Result<Json<User>, StatusCode> {
    repository
        .user(*id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: u32,
    pub name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn user(&self, id: u32) -> Option<User>;
}

impl FromRef<AppState> for Arc<dyn UserRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.users.clone()
    }
}

#[async_trait]
impl UserRepository for Arc<dyn UserRepository> {
    async fn user(&self, id: u32) -> Option<User> {
        (**self).user(id).await
    }
}

#[async_trait]
impl UserRepository for MemoryDb {
    async fn user(&self, id: u32) -> Option<User> {
        self.users.get(&id).cloned()
    }
}

#[async_trait]
impl UserRepository for SqliteDb {
    async fn user(&self, id: u32) -> Option<User> {
        sqlx::query_as("SELECT id, name FROM Users WHERE id = ?")
            .bind(id)
            .fetch_one(self)
            .await
            .ok()
    }
}
