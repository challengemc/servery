mod api;
mod db;
mod server;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let config: Config = toml::from_str(&fs::read_to_string("servery.toml").await?)?;

    let db = mongodb::Client::with_uri_str(&config.db_uri)
        .await?
        .database(&config.app_name);
    api::run(config.app_name, db.collection_with_type("server")).await
}

#[derive(Deserialize)]
struct Config {
    app_name: String,
    db_uri: String,
}
