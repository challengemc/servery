use crate::db::ServerDb;
use anyhow::Result;
use bollard::{container::CreateContainerOptions, Docker};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub mods: Vec<Url>,
}

#[derive(Serialize, Deserialize)]
pub struct NewServer {
    pub name: String,
    pub mods: Vec<Url>,
}

impl NewServer {
    pub async fn create(self, db: &impl ServerDb) -> Result<ObjectId> {
        let id = db.insert(self).await?;
        Ok(id)
    }
}
