use crate::values::DEFAULT_RFS_DIR;

#[derive(Default, Clone)]
pub struct FSConfig {
    pub home_dir: String,
    pub rfs_dir: String,
    pub metafiles_dir: String,
    pub file_parts_dir: String,
    pub files_dir: String,
}

impl FSConfig {
    pub fn new(rfs_dir: Option<String>) -> Self {
        let rfs_dir = rfs_dir.unwrap_or(DEFAULT_RFS_DIR.to_string());
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
        let rfs_dir = home_dir.clone() + "/" + &rfs_dir;
        let metafiles_dir = rfs_dir.clone() + "/metafiles";
        let file_parts_dir = rfs_dir.clone() + "/file_parts";
        let files_dir = rfs_dir.clone() + "/files";
        FSConfig {
            home_dir,
            rfs_dir,
            metafiles_dir,
            file_parts_dir,
            files_dir,
        }
    }
}