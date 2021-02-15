use crate::server::Server;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use tokio::{
    fs::{self, File},
    io::{self, AsyncWriteExt},
    sync::RwLock,
};

pub type ServerDb = Arc<RwLock<JsonServerDb>>;
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub async fn load<P: AsRef<Path>>(path: P) -> Result<ServerDb> {
    Ok(RwLock::new(JsonServerDb::load(path).await?).into())
}

pub struct JsonServerDb {
    path: PathBuf,
    servers: Vec<Server>,
}

impl JsonServerDb {
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_owned(),
            servers: serde_json::from_str(&fs::read_to_string(path).await?)?,
        })
    }

    pub fn all(&self) -> &[Server] {
        &self.servers
    }

    pub fn by_id(&self, id: u32) -> Option<&Server> {
        self.servers.iter().find(|server| server.id == id)
    }

    pub async fn new_id(&self) -> u32 {
        (0..)
            .find(|&id| self.by_id(id).is_none())
            .expect("No id available") // Should never happen
    }

    pub async fn add(&mut self, server: Server) -> Result<()> {
        if self.by_id(server.id).is_some() {
            return Err(Error::DuplicateId(server.id));
        }
        self.servers.push(server);
        File::create(&self.path)
            .await?
            .write_all(&serde_json::to_vec(&self.servers)?)
            .await?;
        Ok(())
    }
}
#[derive(Error, Debug)]
pub enum Error {
    #[error("Server with ID {0} already exists")]
    DuplicateId(u32),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}
