pub mod handlers;
pub mod models;
pub mod server;
pub mod storage;
pub mod utils;

use server::{ServerConfig, init_logging, serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let config = ServerConfig::from_env();
    serve(config).await
}
