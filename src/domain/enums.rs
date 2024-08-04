use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PieceDownloadStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
}
