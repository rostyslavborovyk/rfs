use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: String,
    pub name: String,
    pub length: u64,
    pub peers: Vec<String>,
    pub pieceSize: u64,
    pub hashes: Vec<String>,
}
