use std::fs;
use crate::domain::config::FSConfig;

fn check_folder(path: &str) {
    if let Err(_) = fs::read_dir(path) {
        if let Err(err) = fs::create_dir(path) {
            println!("Metafiles dir was not found and unable to create it! {err}")
        };
    };
}

pub fn check_folders(config: &FSConfig) {
    check_folder(&config.rfs_dir);
    check_folder(&config.metafiles_dir);
    check_folder(&config.files_dir);
    check_folder(&config.file_parts_dir);
}