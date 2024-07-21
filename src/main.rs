use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;
use distributed_fs::peer::client::{Client};
use distributed_fs::peer::state_container::{StateContainer};

#[tokio::main]
async fn main() {
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));
    
    let mut client = Client::new("127.0.0.1:8003".to_string(), sharable_state_container.clone());
    
    client.load_file("meta_files/image.HEIC.json").await.unwrap();
    
    let start = Instant::now();
    client.download_file(String::from("a8f106a9-0066-4946-b691-49721c94d615")).await.unwrap();
    println!("Time elapsed: {}ms", start.elapsed().as_millis())

    // tokio::spawn(async move {
    //     serve_listener(
    //         String::from("127.0.0.1:8001"),
    //         &mut sharable_state_container.clone(),
    //     ).await;
    // });
}
