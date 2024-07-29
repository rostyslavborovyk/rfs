use std::os::macos::fs::MetadataExt;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::sync::Mutex;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::listener::serve_listener;
use distributed_fs::peer::state_container::StateContainer;


#[tokio::test]
async fn main() {
    // todo: create necessary files and folders (i.e meta_files/metafile, files/file) and clean up them

    // setting up the peer
    let peer_address = "127.0.0.1:8001".to_string();
    let host_address = "127.0.0.1:8003".to_string();

    let peer_sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));

    let mut peer_client = Client::new(peer_address.clone(), peer_sharable_state_container.clone());
    peer_client.load_state(peer_address.clone()).await.unwrap();

    tokio::spawn(async move {
        serve_listener(
            peer_address,
            &mut peer_sharable_state_container.clone(),
        ).await;
    });

    // setting up the client
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));

    let mut client = Client::new(host_address.clone(), sharable_state_container.clone());
    client.load_state(host_address).await.unwrap();
    client.download_file(String::from("0155d08b-609b-45fa-804d-53456c2a863d")).await.unwrap();

    let resulting_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("files/new-image.HEIC")
        .await
        .map_err(|err| format!("Error when opening a file {err}")).unwrap();
    let m = resulting_file.metadata().await.unwrap();

    assert_eq!(m.st_size(), 1121518);
}
