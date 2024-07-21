use serde::{Deserialize, Serialize};
use crate::peer::models::File;
use tokio::fs;


#[derive(Serialize, Deserialize)]
pub struct RFSFile {
    pub data: File
}

impl RFSFile {
    pub async fn from_path(path: &str) -> Self {
        let contents = fs::read(path).await.unwrap();
        let data: File = serde_json::from_slice(contents.as_slice()).unwrap();
        RFSFile {
            data,
        }
    }

    pub async fn save(&self) -> Result<(), String>{
        let path = String::from("meta_files/") 
            + &self.data.name.split('.').next()
            .ok_or("Failed to parse the file name, should be in format {name}.{extension}!")? 
            + ".rfs";
        let contents = serde_json::to_string(&self.data).unwrap();
        fs::write(path, contents).await.unwrap();
        Ok(())
    }

    pub fn get_path(&self) -> String {
        "files/".to_string() + &self.data.name
    }
}
