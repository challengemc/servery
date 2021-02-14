mod api;
mod db;
mod server;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    api::run().await
}
