use crate::peer::models::File;
use tokio::fs;

pub struct RFSFile {
    pub data: File
}

impl RFSFile {
    pub async fn from_path(path: String) -> Self {
        let contents = fs::read(path).await.unwrap();
        let data: File = serde_json::from_slice(contents.as_slice()).unwrap();
        RFSFile {
            data,
        }
    }
}