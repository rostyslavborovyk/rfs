use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: String,
    pub hash: String,
    pub name: String,
    pub length: u64,
    pub peers: Vec<String>,  // todo: rename to seeds
    pub piece_size: u64,
    pub hashes: Vec<String>,
}
