use std::sync::Arc;
use tokio::sync::Mutex;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::state_container::StateContainer;


#[tokio::main]
async fn main() {
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));
    
    let client = Arc::new(Client::new("127.0.0.1:8000".to_string(), sharable_state_container.clone()));
    
    client.generate_meta_file("files/image.HEIC").await.unwrap();
    println!("Finished!")
}
