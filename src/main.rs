#![deny(nonstandard_style, rust_2018_idioms)]

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
    dotenv::dotenv()?;
    pretty_env_logger::init_timed();

    let config: Config =
        toml::from_str(&fs::read_to_string(include_config!("servery.toml")).await?)?;
    include_config!("fabric.toml");

    let db = mongodb::Client::with_uri_str(&config.db_uri)
        .await?
        .database(&config.app_name);
    api::run(
        AppState {
            name: config.app_name,
        },
        db.collection_with_type("server"),
    )
    .await
}

#[derive(Deserialize)]
struct Config {
    app_name: String,
    db_uri: String,
}

pub struct AppState {
    pub name: String,
}

/// Writes a default file included from `$CARGO_MANIFEST_DIR/assets/$file` to the
/// current working directory if it does not exist.
#[macro_export]
macro_rules! include_config {
    ($file:expr $(,)?) => {{
        let path = Path::new($file);
        if !path.exists() {
            info!("Writing default {:?}", path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            File::create(path)
                .await?
                .write_all(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/",
                    $file
                )))
                .await?;
        }
        path
    }};
}
