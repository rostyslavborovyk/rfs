use base64::Engine;
use base64::engine::general_purpose;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::peer::file::RFSFile;
use crate::domain::models::File;
use crate::values::DEFAULT_PIECE_SIZE;

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
            }
        })
}