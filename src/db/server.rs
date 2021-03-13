use crate::server::{NewServer, Server};
use anyhow::Result;
use async_trait::async_trait;
use bson::{doc, oid::ObjectId};
use mongodb::{Collection, Cursor};
use std::error::Error;
use tokio_stream::Stream;

#[async_trait]
pub trait ServerDb: Clone + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;
    type AllStream: Stream<Item = Result<Server, Self::Error>> + Send;

    async fn all(&self) -> Result<Self::AllStream, Self::Error>;

    async fn by_id(&self, id: &ObjectId) -> Result<Option<Server>, Self::Error>;

    async fn insert(&self, server: NewServer) -> Result<ObjectId, Self::Error>;
}

#[async_trait]
impl ServerDb for Collection<Server> {
    type Error = mongodb::error::Error;
    type AllStream = Cursor<Server>;

    async fn all(&self) -> Result<Self::AllStream, Self::Error> {
        Ok(self.find(None, None).await?)
    }

    async fn by_id(&self, id: &ObjectId) -> Result<Option<Server>, Self::Error> {
        self.find_one(doc! {"_id": id}, None).await
    }

    async fn insert(&self, server: NewServer) -> Result<ObjectId, Self::Error> {
        let id = ObjectId::new();
        let server = Server {
            id: id.clone(),
            name: server.name,
            version: server.version,
            mods: server.mods,
        };
        self.insert_one(server, None).await?;
        Ok(id)
    }
}
