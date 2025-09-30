use crate::{handlers, storage::Storage};
use axum::{
    routing::{get, post},
    Router,
};
use std::{env, net::SocketAddr, path::PathBuf};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, Level};

pub struct ServerConfig {
    pub port: String,
    pub data_dir: PathBuf,
    pub max_reqs: usize,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let port = env::var("CATCHHOOK_PORT").unwrap_or_else(|_| "43999".into());
        let data_dir = PathBuf::from(
            env::var("CATCHHOOK_DATA").unwrap_or_else(|_| "./catchhook-data".into()),
        );
        let max_reqs: usize = env::var("CATCHHOOK_MAX_REQS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_000);

        Self {
            port,
            data_dir,
            max_reqs,
        }
    }
}

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with_max_level(Level::INFO)
        .init();
}

pub fn create_router(storage: Storage) -> Router {
    let static_service = ServeDir::new("./public").append_index_html_on_directories(true);

    Router::new()
        .route("/health", get(handlers::health))
        .route("/webhook", post(handlers::post_webhook))
        .route("/latest", get(handlers::get_latest))
        .route("/req/{id}", get(handlers::get_one))
        .fallback_service(static_service)
        .with_state(storage)
        .layer(TraceLayer::new_for_http())
}

pub async fn serve(config: ServerConfig) -> anyhow::Result<()> {
    let storage = Storage::new(&config.data_dir, config.max_reqs)?;
    let app = create_router(storage);

    let addr: SocketAddr = format!("0.0.0.0:{}", config.port)
        .parse()
        .expect("Invalid address format");
    
    info!("🚀 Catchhook running at http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
