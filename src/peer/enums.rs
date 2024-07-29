#[derive(PartialEq)]
pub enum ConnectionState {
    Connected,
    InfoRetrieved,
}

#[derive(Clone, Debug)]
pub enum FileManagerFileStatus {
    Downloaded,
    NotDownloaded,
}