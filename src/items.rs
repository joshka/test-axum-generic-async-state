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
    Router::new().route("/{id}", get(item::<Arc<dyn ItemRepository>>))
}

async fn item<R: ItemRepository>(
    id: Path<u32>,
    repository: State<R>,
) -> Result<Json<Item>, StatusCode> {
    repository
        .item(*id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
}

impl Item {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}

#[async_trait]
pub trait ItemRepository: Send + Sync + 'static {
    async fn item(&self, id: u32) -> Option<Item>;
}

impl FromRef<AppState> for Arc<dyn ItemRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.items.clone()
    }
}

#[async_trait]
impl ItemRepository for Arc<dyn ItemRepository> {
    async fn item(&self, id: u32) -> Option<Item> {
        (**self).item(id).await
    }
}

#[async_trait]
impl ItemRepository for MemoryDb {
    async fn item(&self, id: u32) -> Option<Item> {
        self.items.get(&id).cloned()
    }
}

#[async_trait]
impl ItemRepository for SqliteDb {
    async fn item(&self, id: u32) -> Option<Item> {
        sqlx::query_as("SELECT id, name FROM Items WHERE id = ?")
            .bind(id)
            .fetch_one(self)
            .await
            .ok()
    }
}
