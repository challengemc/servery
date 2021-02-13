use crate::server::Server;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

#[async_trait]
pub trait ServerDb {
    async fn all(&self) -> Result<Vec<Server>>;
}

#[async_trait]
impl ServerDb for SqlitePool {
    async fn all(&self) -> Result<Vec<Server>> {
        Ok(sqlx::query_as!(Server, "SELECT * FROM server")
            .fetch_all(self)
            .await?)
    }
}
