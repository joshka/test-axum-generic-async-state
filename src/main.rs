use std::{collections::HashMap, io, net::Ipv4Addr, sync::Arc};

use axum::{
    Router,
    extract::{FromRef, Path, State},
    http::StatusCode,
    routing::get,
};
use sqlx::prelude::FromRow;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = (Ipv4Addr::LOCALHOST, 32123);
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!("Listening on: http://{local_addr}");
    let repository = InMemoryDb::new();
    let state = AppState { repository };
    let router = router(state);
    axum::serve(listener, router).await
}

// This compiles
fn router(state: AppState<InMemoryDb>) -> Router {
    Router::new()
        .route("/user/{id}", get(get_user::<InMemoryDb>))
        .with_state(state)
}

// This does not compile
fn router_generic<R: UserRepository>(state: AppState<R>) -> Router {
    Router::new()
        .route("/user/{id}", get(get_user::<R>))
        .with_state(state)
}

async fn get_user<R: UserRepository>(
    id: Path<u32>,
    repository: State<R>,
) -> Result<String, StatusCode> {
    repository
        .get_user(*id)
        .await
        .map(|user| user.name.to_string())
        .ok_or(StatusCode::NOT_FOUND)
}

trait UserRepository: Clone + Send + Sync + 'static {
    async fn get_user(&self, id: u32) -> Option<User>;
}

#[derive(Clone)]
struct AppState<R: UserRepository> {
    repository: R,
}

impl FromRef<AppState<InMemoryDb>> for InMemoryDb {
    fn from_ref(state: &AppState<InMemoryDb>) -> Self {
        state.repository.clone()
    }
}

impl FromRef<AppState<SqlDb>> for SqlDb {
    fn from_ref(state: &AppState<SqlDb>) -> Self {
        state.repository.clone()
    }
}

#[derive(Clone)]
struct InMemoryDb {
    users: Arc<HashMap<u32, User>>,
}

impl UserRepository for InMemoryDb {
    async fn get_user(&self, id: u32) -> Option<User> {
        self.users.get(&id).cloned()
    }
}

#[derive(Clone)]
struct SqlDb {
    pool: sqlx::SqlitePool,
}

impl UserRepository for SqlDb {
    async fn get_user(&self, _id: u32) -> Option<User> {
        sqlx::query_as("SELECT id, name FROM Users WHERE id = ?")
            .bind(_id)
            .fetch_one(&self.pool)
            .await
            .ok()
    }
}

#[derive(Debug, Clone, FromRow)]
struct User {
    id: u32,
    name: String,
}

impl User {
    fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}

impl InMemoryDb {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User::new(1, "foo".to_string()));
        users.insert(2, User::new(2, "bar".to_string()));
        Self {
            users: Arc::new(users),
        }
    }
}
