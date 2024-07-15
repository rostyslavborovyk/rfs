use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc};
use crate::peer::connection::Connection;
use crate::peer::enums::FileManagerFileStatus;
use crate::peer::file::RFSFile;
use crate::utils::get_now;
use futures::future::join_all;
use tokio::net::TcpListener;
use crate::peer::state_container::{SharableStateContainer, StateContainer};

pub struct LocalFSInfo {

}

pub struct FileManagerFile {
    file: RFSFile,
    last_sync_with_local_fs: u128,
    status: FileManagerFileStatus,
}

pub struct FileManager {
    files: HashMap<String, FileManagerFile>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: Default::default(),
        }
    }
    
    pub fn add_file(&mut self, file: RFSFile) {
        // todo: check if file with this name and piece hashes already present in the system
        let file_id = file.data.file.id.clone();
        let file_ = FileManagerFile {
            file,
            last_sync_with_local_fs: get_now(),
            status: FileManagerFileStatus::NotDownloaded,
        };
        self.files.insert(file_id, file_);
    }
    
    pub async fn download_file(&self, file_id: String) {
        let file = match self.files.get(&file_id) {
            None => {
                return;
            }
            Some(v) => v
        };
        
        let peers = file.file.data.peers.clone();
        
        let connections: Vec<Connection> = join_all(peers.iter().map(|addr| async {
            Connection::from_address(addr).await
        })).await;
        
        for mut c in connections {
            c.retrieve_info().await;
        };
        
        // using ping values determine how many pieces should be downloaded from each peer
        
        // download file pieces
        
        // check pieces hashes
        
        // assemble pieces into a file 
    }
}

pub struct Client {
    pub state_container: SharableStateContainer
}

impl Client {
    pub fn new(state_container: SharableStateContainer) -> Self {
        Client {
            state_container,
        }
    }
    
    async fn process(&mut self) {
        println!("Processed!")
    }

    async fn load_file(&mut self, path: String) -> Result<(), Box<dyn Error>> {
        let file = RFSFile::from_path(path).await;
        // self.file_manager.add_file(file);
        Ok(())
    }
    
    
}

pub fn hello() {
    println!("Hello");
}