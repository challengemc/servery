mod api;
mod db;
mod server;

use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::path::Path;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let config_path = Path::new("servery.toml");
    if !config_path.exists() {
        info!("Writing default config to {:?}", config_path);
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        File::create(config_path)
            .await?
            .write_all(include_bytes!("../default_config.toml"))
            .await?;
    }
    let config: Config = toml::from_str(&fs::read_to_string(config_path).await?)?;

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
