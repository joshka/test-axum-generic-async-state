use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

use axum::Router;
use memory_db::MemoryDb;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::level_filters::LevelFilter;

use self::items::ItemRepository;
use self::users::UserRepository;

mod items;
mod memory_db;
mod sqlite;
mod users;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let addr = (Ipv4Addr::LOCALHOST, 32123);
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!("Listening on: http://{local_addr}");
    let repository = Arc::new(MemoryDb::new());
    let state = AppState {
        users: repository.clone(),
        items: repository.clone(),
    };
    let router = router(state);
    axum::serve(listener, router).await
}

fn router(state: AppState) -> Router {
    Router::new()
        .nest("/user", users::routes())
        .nest("/item", items::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Clone)]
struct AppState {
    users: Arc<dyn UserRepository>,
    items: Arc<dyn ItemRepository>,
}
