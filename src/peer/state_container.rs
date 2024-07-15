use std::collections::HashMap;
use std::sync::Arc;
use crate::peer::client::{FileManager, LocalFSInfo};
use tokio::sync::{Mutex};

pub type SharableStateContainer = Arc<Mutex<StateContainer>>;

pub struct StateContainer {
    pub peer_addresses: Vec<String>,
    pub local_fs_info: LocalFSInfo,
    pub file_manager: FileManager,
}


impl StateContainer {
    pub fn new() -> Self {
        StateContainer {
            peer_addresses: vec![],
            local_fs_info: LocalFSInfo{},
            file_manager: FileManager::new(),
        }
    }
}


