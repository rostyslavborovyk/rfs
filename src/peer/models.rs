use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FileMeta {
    pub id: String,
    pub name: String,
    pub length: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Piece {
    pub size: u64,
    pub hashes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub file: FileMeta,
    pub peers: Vec<String>,
    pub piece: Piece,
}
