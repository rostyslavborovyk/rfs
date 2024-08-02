use eframe::egui::Color32;

pub const DEFAULT_PIECE_SIZE: u64 = 2u64.pow(14);
pub const DEFAULT_BUFFER_SIZE: usize = 2usize.pow(16);
pub const SYNC_DELAY_SECS: u64 = 1;
// pub const LOCAL_PEER_ADDRESS: &str = "127.0.0.1:8000";
// pub const DEFAULT_RFS_DIR: &str = ".rfs_peer2";
pub const LOCAL_PEER_ADDRESS: &str = "127.0.0.1:8001";
pub const DEFAULT_RFS_DIR: &str = ".rfs";

pub const ACCENT: Color32 = Color32::from_rgb(200, 255, 200);
