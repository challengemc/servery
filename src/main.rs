mod api;
mod db;
mod server;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    api::run().await
}
