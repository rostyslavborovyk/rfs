use std::collections::HashSet;
use crate::peer::file::RFSFile;
use crate::peer::state::{KnownPeer, SharableStateContainer};
use tokio::fs;
use crate::domain::config::FSConfig;
use crate::domain::files::generate_meta_file;

#[derive(Clone)]
pub struct LocalFSInfo {}

pub struct Client {
    pub address: String,
    pub state_container: SharableStateContainer,
}

impl Client {
    pub fn new(address: String, state_container: SharableStateContainer) -> Self {
        Client {
            address,
            state_container,
        }
    }

    pub async fn generate_meta_file(&self, path: &str) -> Result<(), String> {
        let rfs_file = generate_meta_file(self.address.clone(), path)?;
        rfs_file.save_to_project_dir().await?;
        Ok(())
    }

pub async fn load_state(&mut self, own_address: String, fs_config: &FSConfig) -> Result<(), String> {
        self.load_metafiles(fs_config).await?;
        self.set_known_peers_from_files(own_address).await?;
        Ok(())
    }
    
    pub async fn load_metafiles(&mut self, fs_config: &FSConfig) -> Result<(), String> {
        let mut locked_state_container = self.state_container.lock().await;
        let mut entries = fs::read_dir(fs_config.metafiles_dir.clone()).await.unwrap();
        while let Some(entry) = entries.next_entry().await.map_err(|_| "Failed to read entry")? {
            let path = entry.path();
            let path = path.to_str().unwrap();
            if path.split('.').last() == Some("rfs") {
                let file = RFSFile::from_path(path).await;
                locked_state_container.file_manager.add_file(file);
            }
        }
        Ok(())
    }

    pub async fn set_known_peers_from_files(&self, own_address: String) -> Result<(), String> {
        let mut locked_state_container = self.state_container.lock().await;
        let mut peers: HashSet<String> = HashSet::new();
        for file in locked_state_container.file_manager.get_files() {
            for peer in file.data.peers {
                if !peer.eq(&own_address) {
                    peers.insert(peer);
                }
            }
        }
        locked_state_container.known_peers =
            peers.into_iter().map(|address| KnownPeer { address, ping: None }).collect();
        Ok(())
    }

    pub async fn download_file(&mut self, file_id: String) -> Result<(), String> {
        let locked_state_container = self.state_container.lock().await;
        locked_state_container.file_manager.download_file(file_id).await?;
        Ok(())
    }
}
