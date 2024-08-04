use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::peer::client::{LocalFSInfo};
use tokio::sync::{Mutex};
use crate::domain::config::FSConfig;
use crate::domain::enums::PieceDownloadStatus;
use crate::peer::file::FileManager;

pub type SharableStateContainer = Arc<Mutex<State>>;

#[derive(Clone)]
pub struct PieceDownloadProgress {
    pub piece: u64,
    pub status: PieceDownloadStatus,
}

impl PieceDownloadProgress {
    pub fn empty(piece: u64) -> Self {
        Self {
            piece,
            status: PieceDownloadStatus::NotDownloaded,
        }
    }
}

pub struct FileDownloadProgress {
    pub pieces: Vec<PieceDownloadProgress>
}

impl FileDownloadProgress {
    pub fn empty(pieces: u64) -> Self {
        Self {
            pieces: (0..pieces).map(|p| PieceDownloadProgress::empty(p)).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KnownPeer {
    pub address: String,
    pub ping: Option<i64>,
}

impl KnownPeer {
    pub fn accessible(&self) -> bool {
        self.ping.is_some()
    }
}

pub struct State {
    pub known_peers: Vec<KnownPeer>,
    pub local_fs_info: LocalFSInfo,
    pub file_manager: FileManager,
}

impl State {
    pub fn new(fs_config: FSConfig) -> Self {
        State {
            known_peers: vec![],
            local_fs_info: LocalFSInfo{},
            file_manager: FileManager::new(fs_config),
        }
    }
    
    pub fn update_pings_for_peers(&mut self, values: Vec<KnownPeer>) {
        for value in values {
            if let Some(peer) = self.known_peers.iter_mut().find(|p| p.address.eq(&value.address)) {
                peer.ping = value.ping;
            };
        }
    }
}
