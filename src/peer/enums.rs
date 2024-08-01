use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum ConnectionState {
    Connected,
    InfoRetrieved,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileStatus {
    Downloaded,
    NotDownloaded,
}