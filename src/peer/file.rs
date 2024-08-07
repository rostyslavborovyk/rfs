use std::collections::HashMap;
use tokio::fs;
use futures::future::join_all;
use tokio;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use crate::domain::config::FSConfig;
use crate::domain::enums::PieceDownloadStatus;
use crate::domain::files::RFSFile;
use crate::peer::connection::{Connection, FilePieceResponseFrame};
use crate::peer::connection::ConnectionFrame::FilePieceDownloadStatusResponse;

pub struct FileManager {
    files: HashMap<String, RFSFile>,
    fs_config: FSConfig
}

impl FileManager {
    pub async fn get_file_piece(&mut self, file_id: String, piece: u64) -> Result<Vec<u8>, String> {
        let file = self.files.get(&file_id).ok_or(format!("File not found by id {:?}", file_id))?;

        // todo: rewrite to read only piece from the fs
        let contents = tokio::fs::read(file.get_path()).await
            .map_err(|err| format!("Error when reading file {err}"))?;

        let start = (file.data.piece_size * piece) as usize;
        let end = (file.data.piece_size * (piece + 1)) as usize;

        let piece = if end < contents.len() {
            &contents[start..end]
        } else {
            &contents[start..]
        };

        Ok(piece.to_vec())
    }

    pub fn get_files(&self) -> Vec<RFSFile> {
        Vec::from_iter(self.files.values().cloned())
    }

    pub fn get_file_ids(&self) -> Vec<String> {
        Vec::from_iter(self.files.keys().cloned())
    }
}

impl FileManager {
    pub fn new(fs_config: FSConfig) -> Self {
        Self {
            files: Default::default(),
            fs_config,
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
        self.files.insert(file_id, file);
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
            .open(self.fs_config.files_dir.clone() + "/" + &file_name)
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

    pub async fn download_file(&self, ui_connection: &mut Connection, file_id: String) -> Result<(), String> {
        let file = self.files.get(&file_id).ok_or("No file with such name")?;

        let peers = file.data.peers.clone();

        let connections: Vec<Option<Connection>> = Connection::from_addresses(peers).await;

        let connections = join_all(connections.into_iter().flatten().map(|mut c| async {
            if let Err(e) = c.retrieve_info().await {
                println!("Error when retrieving connection info: {:?}", e);
            };
            c
        })).await;

        let pings = connections.iter().map(|c| {
            match &c.info {
                None => u128::MAX,
                Some(info) => info.ping as u128
            }
        }).collect::<Vec<u128>>();

        let pieces_ratios = self.calculate_pieces_ratio(file.data.hashes.len() as i64, pings);
        let assigned_pieces = self.assign_pieces(pieces_ratios);

        let mut piece_ids = vec![];
        for (pieces, mut c) in assigned_pieces.iter().zip(connections) {
            for piece in pieces {
                ui_connection.send_file_piece_download_status(
                    file_id.clone(), piece.to_owned(), PieceDownloadStatus::Downloading,
                ).await;
                println!("Written downloading frame to {file_id}");
                let frame = c.get_file_piece(file_id.clone(), piece.to_owned()).await?;
                // todo: check piece hash before saving
                piece_ids.push(frame.get_piece_id());
                self.save_file_piece(frame).await?;

                ui_connection.send_file_piece_download_status(
                    file_id.clone(), piece.to_owned(), PieceDownloadStatus::Downloaded,
                ).await;
                println!("Written downloaded frame to {file_id}");
            }
        };

        self.assemble_file(file.data.name.clone(), piece_ids).await?;
        Ok(())
    }
}