use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions};
use std::{env, error::Error, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;

    println!("cargo:rerun-if-changed=migrations");

    let url = env::var("DATABASE_URL")?;
    sqlx::migrate!()
        .run(
            &mut SqliteConnectOptions::from_str(&url)?
                .create_if_missing(true)
                .connect()
                .await?,
        )
        .await?;
    Ok(())
}
