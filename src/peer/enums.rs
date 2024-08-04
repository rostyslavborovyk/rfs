use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum ConnectionState {
    Connected,
    InfoRetrieved,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FileStatus {
    Downloaded,
    Downloading,
    NotDownloaded,
}