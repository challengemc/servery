use crate::server::{NewServer, Server};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::RwLock,
};

pub type ServerDb = Arc<RwLock<JsonServerDb>>;

pub async fn load<P: AsRef<Path>>(path: P) -> Result<ServerDb> {
    Ok(RwLock::new(JsonServerDb::load(path).await?).into())
}

pub struct JsonServerDb {
    path: PathBuf,
    servers: Vec<Server>,
}

impl JsonServerDb {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            File::create(path).await?.write_all(b"[]").await?;
        }
        Ok(Self {
            path: path.to_owned(),
            servers: serde_json::from_str(&fs::read_to_string(path).await?)?,
        })
    }

    pub fn all(&self) -> &[Server] {
        &self.servers
    }

    pub fn by_id(&self, id: u32) -> Option<&Server> {
        self.servers.iter().find(|server| server.id == id)
    }

    pub async fn add(&mut self, server: NewServer) -> Result<u32> {
        let id = self.new_id();
        self.servers.push(Server {
            id,
            name: server.name,
        });
        File::create(&self.path)
            .await?
            .write_all(&serde_json::to_vec(&self.servers)?)
            .await?;
        Ok(id)
    }

    fn new_id(&self) -> u32 {
        (0..)
            .find(|&id| self.by_id(id).is_none())
            .expect("No id available") // Should never happen
    }
}
