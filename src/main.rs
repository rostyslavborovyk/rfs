use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use distributed_fs::peer::client::{Client, FileManager, hello, LocalFSInfo};
use distributed_fs::peer::connection::{Connection, ConnectionFrame, GetInfoFrame, GetPingFrame, InfoResponseFrame, PingResponseFrame};
use distributed_fs::peer::state_container::{SharableStateContainer, StateContainer};


async fn process_get_ping_frame(
    connection: &mut Connection,
    _: &mut SharableStateContainer,
    _: GetPingFrame,
) {
    connection.write_frame(ConnectionFrame::PingResponse(PingResponseFrame{})).await;
}

async fn process_get_info_frame(
    connection: &mut Connection,
    _: &mut SharableStateContainer,
    _: GetInfoFrame,
) {
    connection.write_frame(ConnectionFrame::InfoResponse(InfoResponseFrame{
        file_ids: vec![],
    })).await;

}


async fn process_inbound_connection(
    connection: &mut Connection,
    sharable_state_container: &mut SharableStateContainer,
) {
    loop {
        match connection.read_frame().await {
            None => {
                eprintln!("Invalid data received!");
                continue;
            }
            Some(info_response) => {
                match info_response {
                    ConnectionFrame::GetPing(frame) => {
                        process_get_ping_frame(connection, sharable_state_container, frame).await
                    },
                    ConnectionFrame::GetInfo(frame) => {
                        process_get_info_frame(connection, sharable_state_container, frame).await
                    },
                    _ => {
                        eprintln!("Wrong frame received!");
                        continue;
                    }
                }
            }
        };
    }
}

async fn serve_listener(
    sharable_state_container: &mut SharableStateContainer,
) {
    let listener = TcpListener::bind("127.0.0.1:8001").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let mut connection = Connection::from_stream(socket).await;
        println!("Accepted");
        let mut sharable_state_container = sharable_state_container.clone();
        tokio::spawn(async move {
            process_inbound_connection(&mut connection, &mut sharable_state_container).await;
        });
    };
}

#[tokio::main]
async fn main() {
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));
    
    let client = Arc::new(Client::new(sharable_state_container.clone()));

    tokio::spawn(async move {
        serve_listener(&mut sharable_state_container.clone()).await;
    });
}
