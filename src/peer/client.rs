use std::collections::HashMap;
use std::error::Error;
use crate::peer::connection::{Connection};
use crate::peer::enums::FileManagerFileStatus;
use crate::peer::file::RFSFile;
use crate::utils::get_now;
use futures::future::join_all;
use crate::peer::state_container::{SharableStateContainer};

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

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            files: Default::default(),
        }
    }

    fn calculate_pieces_ratio(&self, n_pieces: i64, pings: Vec<u128>) -> Vec<i64> {
        let values = pings.into_iter().map(|p| 1f64 / (p as f64)).collect::<Vec<f64>>();
        let sum = values.iter().sum::<f64>();
        let res = values.into_iter().map(|p| ((p / sum) * n_pieces as f64).round() as i64).collect::<Vec<i64>>();
        if res.iter().sum::<i64>() != n_pieces {
            panic!("Pieces ratio doesn't add up: n_pieces = {n_pieces}, res={:?}", res)
        }
        res
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
        
        let connections: Vec<Option<Connection>> = join_all(peers.iter().map(|addr| async move {
            match Connection::from_address(&addr.clone()).await {
                None => {
                    println!("Failed to connect to {:?}", addr);
                    None
                }
                Some(c) => Some(c),
            }
        })).await;

        let connections = connections.into_iter().flatten().collect::<Vec<Connection>>();

        let connections = join_all(connections.into_iter().map(|mut c| async {
            c.retrieve_info().await;
            println!("Connection info: {:?}", c.info);
            c
        })).await;

        let pings = connections.iter().map(move |c| {
            match &c.info {
                None => {
                    u128::MAX
                }
                Some(info) => info.ping
            }
        }).collect::<Vec<u128>>();

        let pieces_ratios = self.calculate_pieces_ratio(
            file.file.data.piece.hashes.len() as i64,
            pings
        );
        
        println!("N pieces {:?}", file.file.data.piece.hashes.len() as i64);
        println!("Pieces ratios for download {:?}", pieces_ratios);

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

    pub async fn load_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = RFSFile::from_path(path).await;
        let mut locked_state_container = self.state_container.lock().await;
        locked_state_container.file_manager.add_file(file);
        Ok(())
    }

    pub async fn download_file(&mut self, file_id: String) -> Result<(), Box<dyn Error>> {
        let locked_state_container = self.state_container.lock().await;
        locked_state_container.file_manager.download_file(file_id).await;
        Ok(())
    }
}

pub fn hello() {
    println!("Hello");
}