use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::peer::client::{LocalFSInfo};
use tokio::sync::{Mutex};
use crate::domain::config::FSConfig;
use crate::peer::file::FileManager;

pub type SharableStateContainer = Arc<Mutex<State>>;

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
