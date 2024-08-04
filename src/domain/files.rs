use std::io::{ErrorKind};
use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::domain::models::File;
use crate::peer::enums::FileStatus;
use crate::values::DEFAULT_PIECE_SIZE;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RFSFile {
    pub data: File,
    pub status: Option<FileStatus>,
}

impl RFSFile {
    pub fn from_path_sync(path: &str) -> Self {
        let contents = std::fs::read(path).unwrap();
        let data: File = serde_json::from_slice(contents.as_slice()).unwrap();
        RFSFile {
            data,
            status: Default::default(),
        }
    }

    pub async fn from_path(path: &str) -> Self {
        let contents = tokio::fs::read(path).await.unwrap();
        let data: File = serde_json::from_slice(contents.as_slice()).unwrap();
        RFSFile {
            data,
            status: Default::default(),
        }
    }

    pub async fn save_to_project_dir(&self) -> Result<(), String>{
        let path = String::from("meta_files/")
            + &self.data.name.split('.').next()
            .ok_or("Failed to parse the file name, should be in format {name}.{extension}!")?
            + ".rfs";
        let contents = serde_json::to_string(&self.data).unwrap();
        tokio::fs::write(path, contents).await.unwrap();
        Ok(())
    }

    pub fn save(&self, path: String) -> Result<(), String>{
        let contents = serde_json::to_string(&self.data).unwrap();
        std::fs::write(path, contents).unwrap();
        Ok(())
    }

    pub fn get_path(&self) -> String {
        "files/".to_string() + &self.data.name
    }
}


pub fn generate_meta_file(host_address: String, path: &str) -> Result<RFSFile, String> {
    let name = path
        .split('/').last().ok_or("Unable to get name from path!")?
        .to_owned();
    let contents = std::fs::read(path)
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
            },
            status: Default::default(),
        })
}

pub fn refresh_file_status(file: &mut RFSFile, files_dir: String) {
    match std::fs::read(files_dir + "/" + &file.data.name) {
        Ok(_) => {
            // todo: check if hash matches
            file.status = Some(FileStatus::Downloaded);
        },
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    if file.status != Some(FileStatus::Downloading) {
                        file.status = Some(FileStatus::NotDownloaded);
                    }
                }
                err => {
                    println!("Unhandled error {err}")
                }
            }
        }
    };
}
