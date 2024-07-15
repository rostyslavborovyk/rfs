#[derive(PartialEq)]
pub enum ConnectionState {
    Connected,
    InfoRetrieved,
}

pub enum FileManagerFileStatus {
    Downloaded,
    NotDownloaded,
}