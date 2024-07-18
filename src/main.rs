use std::sync::Arc;
use tokio::sync::Mutex;
use distributed_fs::peer::client::{Client};
use distributed_fs::peer::state_container::{StateContainer};

#[tokio::main]
async fn main() {
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));
    
    let mut client = Client::new("127.0.0.1:8003".to_string(), sharable_state_container.clone());
    
    client.load_file("meta_files/1.json").await.unwrap();
    
    client.download_file(String::from("b2af0093-4f31-4519-8e97-0940e9973247")).await.unwrap();
    println!("Completed!")

    // tokio::spawn(async move {
    //     serve_listener(
    //         String::from("127.0.0.1:8001"),
    //         &mut sharable_state_container.clone(),
    //     ).await;
    // });
}
