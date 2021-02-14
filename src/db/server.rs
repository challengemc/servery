use crate::server::Server;
use anyhow::Result;
use std::{
    borrow::Borrow,
    ops::Deref,
    path::{Path, PathBuf},
    slice,
    sync::Arc,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::{RwLock, RwLockReadGuard},
};

#[derive(Clone)]
pub struct ServerDb(Arc<RwLock<JsonServerDb>>);

impl ServerDb {
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self(RwLock::new(JsonServerDb::load(path).await?).into()))
    }

    pub async fn all(&self) -> AllServers<'_> {
        AllServers(self.0.read().await)
    }
}

struct JsonServerDb {
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
        self.servers.push(server);
        File::create(&self.path)
            .await?
            .write_all(&serde_json::to_vec(&self.servers)?)
            .await?;
        Ok(())
    }
}

pub struct AllServers<'a>(RwLockReadGuard<'a, JsonServerDb>);

impl<'a> Deref for AllServers<'a> {
    type Target = [Server];

    fn deref(&self) -> &Self::Target {
        self.0.all()
    }
}

impl<'a> Borrow<[Server]> for AllServers<'a> {
    fn borrow(&self) -> &[Server] {
        &**self
    }
}

impl<'a> AsRef<[Server]> for AllServers<'a> {
    fn as_ref(&self) -> &[Server] {
        &**self
    }
}
