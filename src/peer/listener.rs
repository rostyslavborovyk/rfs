use tokio::net::TcpListener;
use crate::peer::connection::{
    Connection,
    ConnectionFrame,
    FilePieceResponseFrame,
    GetFilePieceFrame,
    GetInfoFrame,
    GetPingFrame,
    InfoResponseFrame,
    PingResponseFrame,
};
use crate::peer::state_container::SharableStateContainer;

async fn process_get_ping_frame(
    connection: &mut Connection,
    _: &mut SharableStateContainer,
    _: GetPingFrame,
) {
    connection.write_frame(ConnectionFrame::PingResponse(PingResponseFrame {})).await;
}

async fn process_get_info_frame(
    connection: &mut Connection,
    _: &mut SharableStateContainer,
    _: GetInfoFrame,
) {
    connection.write_frame(ConnectionFrame::InfoResponse(InfoResponseFrame {
        file_ids: vec![],
    })).await;
}

async fn process_get_file_piece_frame(
    connection: &mut Connection,
    container: &mut SharableStateContainer,
    frame: GetFilePieceFrame,
) -> Result<(), String> {
    let mut container_locked = container.lock().await;
    let content = container_locked.file_manager.get_file_piece(frame.file_id.clone(), frame.piece).await?;
    connection.write_frame(ConnectionFrame::FilePieceResponse(FilePieceResponseFrame {
        file_id: frame.file_id,
        piece: frame.piece,
        content,
    })).await;
    Ok(())
}


// todo: rewrite with some pattern?
async fn process_inbound_connection(
    connection: &mut Connection,
    sharable_state_container: &mut SharableStateContainer,
) -> Result<(), String> {
    loop {
        println!("Waiting from new frames...");
        match connection.read_frame().await? {
            ConnectionFrame::GetPing(frame) => {
                process_get_ping_frame(connection, sharable_state_container, frame).await
            }
            ConnectionFrame::GetInfo(frame) => {
                process_get_info_frame(connection, sharable_state_container, frame).await
            }
            ConnectionFrame::GetFilePiece(frame) => {
                process_get_file_piece_frame(connection, sharable_state_container, frame).await?
            }
            _ => {
                eprintln!("Wrong frame received!");
                continue;
            }
        }
    };
}

pub async fn serve_listener(
    addr: String,
    sharable_state_container: &mut SharableStateContainer,
) {
    println!("Serving listener with address {addr}");
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        println!("Waiting for new connection...");
        let (socket, addr) = listener.accept().await.unwrap();
        println!("Accepted new connection from addr {addr}");
        let mut connection = Connection::from_stream(socket).await;
        let mut sharable_state_container = sharable_state_container.clone();
        tokio::spawn(async move {
            process_inbound_connection(&mut connection, &mut sharable_state_container)
                .await.map_err(|err| {
                    println!("Error when processing inbound connection: {err}");
                });
        });
    };
}