use std::sync::Arc;
use tokio::sync::Mutex;
use distributed_fs::domain::config::FSConfig;
use distributed_fs::domain::fs::check_folders;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::connection::{Connection, ConnectionFrame, GetFilePieceFrame};
use distributed_fs::peer::state::State;


#[tokio::main]
async fn main() -> Result<(), String> {
    let start = tokio::time::Instant::now();

    let fs_config = FSConfig::new(None);
    check_folders(&fs_config);

    let sharable_state_container = Arc::new(Mutex::new(State::new(fs_config.clone())));

    let mut client = Client::new("127.0.0.1:8000".to_string(), sharable_state_container.clone());

    client.load_state("127.0.0.1:8000".to_string(), &fs_config).await.unwrap();

    let file_id = "4148f04f-41e3-4f39-94e8-155bc6dcd3ae".to_string();

    let mut connection = Connection::from_address(&"127.0.0.1:8001".to_string()).await.unwrap();
    for piece in 0..10 {
        connection.write_frame(ConnectionFrame::GetFilePiece(GetFilePieceFrame { file_id: file_id.clone(), piece })).await;
    }

    for _ in 0..10 {
        let frame = connection.read_frame().await?;
        match frame {
            ConnectionFrame::FilePieceResponse(frame) => {
                println!("Received file piece frame {}", frame.piece);
            }
            _ => {}
        }
    }

    println!("Time spent: {}ms", start.elapsed().as_millis());

    Ok(())
}