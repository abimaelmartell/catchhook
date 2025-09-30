mod handlers;
mod models;
mod server;
mod storage;
mod utils;

use server::{ServerConfig, init_logging, serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let config = ServerConfig::from_env();
    serve(config).await
}
