use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;
use distributed_fs::domain::config::FSConfig;
use distributed_fs::domain::fs::check_folders;
use distributed_fs::peer::client::{Client};
use distributed_fs::peer::state::{State};

#[tokio::main]
async fn main() {
    let fs_config = FSConfig::new(None);
    check_folders(&fs_config);
    
    let sharable_state_container = Arc::new(Mutex::new(State::new(fs_config.clone())));
    
    let address = "127.0.0.1:8003".to_string();
    let mut client = Client::new(address.clone(), sharable_state_container.clone());

    client.load_state(address, &fs_config).await.unwrap();

    let start = Instant::now();
    client.download_file(String::from("0155d08b-609b-45fa-804d-53456c2a863d")).await.unwrap();
    println!("Time elapsed: {}ms", start.elapsed().as_millis())
}
