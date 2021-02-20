use crate::server::{NewServer, Server};
use anyhow::Result;
use async_trait::async_trait;
use bson::oid::ObjectId;
use mongodb::{Collection, Cursor};
use std::error::Error;
use tokio_stream::Stream;

#[async_trait]
pub trait ServerDb: Clone + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;
    type AllStream: Stream<Item = Result<Server, Self::Error>> + Send;

    async fn all(&self) -> Result<Self::AllStream, Self::Error>;

    async fn insert(&self, server: NewServer) -> Result<ObjectId, Self::Error>;
}

#[async_trait]
impl ServerDb for Collection<Server> {
    type Error = mongodb::error::Error;
    type AllStream = Cursor<Server>;

    async fn all(&self) -> Result<Self::AllStream, Self::Error> {
        Ok(self.find(None, None).await?)
    }

    async fn insert(&self, server: NewServer) -> Result<ObjectId, Self::Error> {
        let id = ObjectId::new();
        let server = Server {
            id: id.clone(),
            name: server.name,
            mods: server.mods,
        };
        self.insert_one(server, None).await?;
        Ok(id)
    }
}
