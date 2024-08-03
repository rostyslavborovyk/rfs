use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum ConnectionStatus {
    Connected,
    InfoRetrieved,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileStatus {
    Downloaded,
    Downloading,
    NotDownloaded,
}