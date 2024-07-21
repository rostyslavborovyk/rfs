use std::sync::Arc;
use tokio::sync::Mutex;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::connection::{Connection, ConnectionFrame, GetFilePieceFrame, GetInfoFrame};
use distributed_fs::peer::state_container::StateContainer;


#[tokio::main]
async fn main() {
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));

    let mut client = Client::new("127.0.0.1:8000".to_string(), sharable_state_container.clone());

    client.load_file("meta_files/image.HEIC.json").await.unwrap();

    let file_id = "BPCry8WD6vjfy51/QYn2cb/fZsfnKIIXCFjDUsHM/wA=".to_string();
    let piece = 0;

    let mut connection = Connection::from_address(&"127.0.0.1:8001".to_string()).await.unwrap();
    connection.write_frame(ConnectionFrame::GetFilePiece(GetFilePieceFrame { file_id, piece })).await;

    let frame = connection.read_frame().await.unwrap();
    
    println!("Received frame {:?}", frame)
}