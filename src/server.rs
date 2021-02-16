use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewServer {
    pub name: String,
}
