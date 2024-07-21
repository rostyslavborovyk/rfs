use std::collections::HashMap;
use std::error::Error;
use base64::Engine;
use base64::engine::general_purpose;
use crate::peer::connection::{Connection, FilePieceResponseFrame};
use crate::peer::enums::FileManagerFileStatus;
use crate::peer::file::RFSFile;
use crate::utils::get_now;
use futures::future::join_all;
use crate::peer::models::File;
use crate::peer::state_container::{SharableStateContainer};
use sha2::{Sha256, Digest};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::values::DEFAULT_PIECE_SIZE;

pub struct LocalFSInfo {}

pub struct FileManagerFile {
    file: RFSFile,
    last_sync_with_local_fs: u128,
    status: FileManagerFileStatus,
}

pub struct FileManager {
    files: HashMap<String, FileManagerFile>,
}

impl FileManager {
    pub async fn generate_meta_file(&self, host_address: String, path: &str) -> Result<RFSFile, String> {
        let name = path.split('/').last().ok_or("Unable to get name from path!")?.to_owned();
        let contents = tokio::fs::read(path).await
            .map_err(|err| format!("Error when reading file {err}"))?;

        let length = contents.len() as u64;
        let mut hasher = Sha256::new();
        hasher.update(&contents);

        let file_id = Uuid::new_v4().to_string();
        let hash = general_purpose::STANDARD.encode(hasher.finalize());

        let hashes: Vec<String> = (0..f64::ceil(contents.len() as f64 / DEFAULT_PIECE_SIZE as f64) as usize)
            .map(|i| {
                let start = i*DEFAULT_PIECE_SIZE as usize;
                let end = (i + 1)*DEFAULT_PIECE_SIZE as usize;
                let piece = if end < contents.len() {
                    &contents[start..end]
                } else {
                    &contents[start..]
                };
                let mut hasher = Sha256::new();
                hasher.update(piece);
                general_purpose::STANDARD.encode(hasher.finalize())
                }
            ).collect();

        Ok(
            RFSFile {
                data: File {
                    id: file_id,
                    hash,
                    name,
                    length,
                    peers: vec![host_address],
                    piece_size: DEFAULT_PIECE_SIZE,
                    hashes,
                }
            })
    }

    pub async fn get_file_piece(&mut self, file_id: String, piece: u64) -> Result<Vec<u8>, String> {
        let file = self.files.get(&file_id).ok_or(format!("File not found by id {:?}", file_id))?;

        // todo: rewrite to read only piece from the fs
        let contents = tokio::fs::read(file.file.get_path()).await
            .map_err(|err| format!("Error when reading file {err}"))?;

        let start = (file.file.data.piece_size * piece) as usize;
        let end = (file.file.data.piece_size * (piece + 1)) as usize;

        let piece = if end < contents.len() {
            &contents[start..end]
        } else {
            &contents[start..]
        };

        Ok(piece.to_vec())
    }
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

    fn calculate_pieces_ratio(&self, n_pieces: i64, pings: Vec<u128>) -> Vec<u64> {
        let values = pings.into_iter().map(|p| 1f64 / (p as f64)).collect::<Vec<f64>>();
        let sum = values.iter().sum::<f64>();
        let res = values.into_iter().map(|p| ((p / sum) * n_pieces as f64).round() as u64).collect::<Vec<u64>>();
        if res.iter().sum::<u64>() != n_pieces as u64 {
            panic!("Pieces ratio doesn't add up: n_pieces = {n_pieces}, res={:?}", res)
        }
        res
    }

    fn assign_pieces(&self, pieces_ratio: Vec<u64>) -> Vec<Vec<u64>> {
        let mut i = 0;
        let mut res = vec![];
        for r in pieces_ratio {
            let mut connection_pieces = vec![];
            for _ in 0..r {
                connection_pieces.push(i);
                i += 1;
            }
            res.push(connection_pieces);
        };
        res
    }

    pub fn add_file(&mut self, file: RFSFile) {
        // todo: check if file with this name and piece hashes already present in the system
        let file_id = file.data.id.clone();
        let file_ = FileManagerFile {
            file,
            last_sync_with_local_fs: get_now(),
            status: FileManagerFileStatus::NotDownloaded,
        };
        self.files.insert(file_id, file_);
    }

    async fn save_file_piece(&self, frame: FilePieceResponseFrame) -> Result<(), String> {
        let path = "file_pieces/".to_string() + &frame.get_piece_id();
        fs::write(path, frame.content).await.map_err(|err| format!("Error when writing file piece {err}"))?;
        Ok(())
    }

    async fn assemble_file(&self, file_name: String, piece_ids: Vec<String>) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("files/new-".to_string() + &file_name)
            .await
            .map_err(|err| format!("Error when opening a file {err}"))?;

        for pid in piece_ids {
            let path = "file_pieces/".to_string() + &pid;
            let contents = tokio::fs::read(&path).await
                .map_err(|err| format!("Error when reading a file piece {err}"))?;
            file.write(&contents)  // or to use write_all?
                .await
                .map_err(|err| format!("Error when writing a file piece {err}"))?;
            tokio::fs::remove_file(path).await
                .map_err(|err| format!("Error when removing a file piece {err}"))?;
        };

        file.flush().await.map_err(|err| format!("Error when flushing a file{err}"))?;
        Ok(())
    }

    pub async fn download_file(&self, file_id: String) -> Result<(), String> {
        let file = self.files.get(&file_id).ok_or("No file with such name")?;

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

        let connections = join_all(connections.into_iter().flatten().map(|mut c| async {
            if let Err(e) = c.retrieve_info().await {
                println!("Error when retrieving connection info: {:?}", e);
            };
            c
        })).await;

        let pings = connections.iter().map(|c| {
            match &c.info {
                None => {
                    u128::MAX
                }
                Some(info) => info.ping
            }
        }).collect::<Vec<u128>>();

        let pieces_ratios = self.calculate_pieces_ratio(
            file.file.data.hashes.len() as i64,
            pings,
        );

        let assigned_pieces = self.assign_pieces(pieces_ratios);

        let mut piece_ids = vec![];
        for (pieces, mut c) in assigned_pieces.iter().zip(connections) {
            for piece in pieces {
                let frame = c.get_file_piece(file_id.clone(), piece.to_owned()).await?;
                // todo: check piece hash before saving
                piece_ids.push(frame.get_piece_id());
                self.save_file_piece(frame).await?;
            }
        };

        self.assemble_file(file.file.data.name.clone(), piece_ids).await?;
        Ok(())
    }
}

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

    pub async fn generate_meta_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let locked_state_container = self.state_container.lock().await;
        let rfs_file = locked_state_container.file_manager.generate_meta_file(self.address.clone(), path).await?;
        rfs_file.save().await;
        Ok(())
    }

    pub async fn load_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = RFSFile::from_path(path).await;
        let mut locked_state_container = self.state_container.lock().await;
        locked_state_container.file_manager.add_file(file);
        Ok(())
    }

    pub async fn download_file(&mut self, file_id: String) -> Result<(), Box<dyn Error>> {
        let locked_state_container = self.state_container.lock().await;
        locked_state_container.file_manager.download_file(file_id).await?;
        Ok(())
    }
}
