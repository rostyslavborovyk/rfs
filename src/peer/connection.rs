use futures::future::join_all;
use serde::{Deserialize, Serialize};
// todo: cbor serialization still produces 31Kb size for the frame with 16Kb of contents. 
// Maybe check some other available formats, or write own binary protocol?
use serde_cbor::{from_slice, to_vec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant};
use crate::domain::enums::PieceDownloadStatus;
use crate::peer::enums::ConnectionState;
use crate::peer::state::KnownPeer;
use crate::values::DEFAULT_BUFFER_SIZE;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetInfoFrame {}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponseFrame {
    pub file_ids: Vec<String>,
    pub known_peers: Vec<KnownPeer>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPingFrame {}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingResponseFrame {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFilePieceFrame {
    pub file_id: String,
    pub piece: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetFileFrame {
    pub file_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilePieceResponseFrame {
    pub file_id: String,
    pub piece: u64,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilePieceDownloadStatusResponseFrame {
    pub file_id: String,
    pub piece: u64,
    pub status: PieceDownloadStatus,
}

impl FilePieceResponseFrame {
    pub fn get_piece_id(&self) -> String {
        self.file_id.clone() + ":" + &self.piece.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum ConnectionFrame {
    #[serde(rename = "GetInfo")]
    GetInfo(GetInfoFrame),

    #[serde(rename = "InfoResponse")]
    InfoResponse(InfoResponseFrame),

    #[serde(rename = "GetPing")]
    GetPing(GetPingFrame),

    #[serde(rename = "PingResponse")]
    PingResponse(PingResponseFrame),

    #[serde(rename = "GetFilePiece")]
    GetFilePiece(GetFilePieceFrame),

    #[serde(rename = "GetFile")]
    GetFile(GetFileFrame),

    #[serde(rename = "FilePieceResponse")]
    FilePieceResponse(FilePieceResponseFrame),

    #[serde(rename = "FilePieceDownloadStatusResponse")]
    FilePieceDownloadStatusResponse(FilePieceDownloadStatusResponseFrame),
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub ping: i64,
    pub file_ids: Vec<String>,
    pub known_peers: Vec<KnownPeer>,
}


// todo: refactor with state pattern https://www.youtube.com/watch?v=_ccDqRTx-JU&t=10s
// define different methods for outbound peer connections and inbound connections
pub struct Connection {
    stream: TcpStream,
    state: ConnectionState,
    pub info: Option<ConnectionInfo>,
    buffer: [u8; DEFAULT_BUFFER_SIZE],
}

impl Connection {
    pub async fn from_address(address: &String) -> Option<Self> {
        match TcpStream::connect(address).await {
            Ok(stream) => Some(
                Connection {
                    stream,
                    state: ConnectionState::Connected,
                    buffer: [0; DEFAULT_BUFFER_SIZE],
                    info: None,
                }
            ),
            Err(err) => { 
                println!("Exception when connecting to the address {}: {}", address, err);
                None
            },
        }
    }

    pub async fn from_addresses(addresses: Vec<String>) -> Vec<Option<Connection>> {
        join_all(addresses.iter().map(|addr| async move {
            Connection::from_address(&addr.clone()).await
        })).await
    }

    pub async fn from_stream(stream: TcpStream) -> Self {
        Connection {
            stream,
            state: ConnectionState::Connected,
            buffer: [0; DEFAULT_BUFFER_SIZE],
            info: None,
        }
    }

    // todo: connection may return more bytes than buffer can load. To rewrite it with an ability
    // for the buffer to store loaded bytes and read from stream again.
    pub async fn read_frame(&mut self) -> Result<ConnectionFrame, String> {
        let size = match self.stream.read_u64().await {
            Ok(v) => Ok(v),
            Err(_) => Err("No bytes received from connection, closing".to_string())
        }?;
        
        if size >= DEFAULT_BUFFER_SIZE as u64 {
            return Err("Frame doesn't fit into the buffer".to_string())
        };

        let frame_buffer = &mut self.buffer[..size as usize];

        let _ = match self.stream.read(frame_buffer).await {
            Ok(0) => {
                Err("No bytes received from connection, closing".to_string())
            }
            Ok(n) => Ok(n),
            Err(e) => {
                Err(format!("Failed to read from socket; err = {:?}", e).to_string())
            }
        }?;

        from_slice(&frame_buffer).map_err(|err| format!("Error when parsing frame {err}"))
    }

    pub async fn write_frame(&mut self, frame: ConnectionFrame) {
        let frame_data = to_vec(&frame).expect("Failed to serialize GetInfo frame!");
        let frame_size: [u8; 8] = (frame_data.len() as u64).to_be_bytes();
        let mut data = Vec::with_capacity(4 + frame_data.len());
        data.extend_from_slice(frame_size.as_ref());
        data.extend_from_slice(frame_data.as_ref());
        println!("Writing frame with size {}", frame_data.len());
        self.stream.write_all(data.as_ref()).await.expect("Failed to send GetInfo frame to the peer");
    }

    pub async fn get_ping(&mut self) -> Result<u128, String> {
        self.write_frame(ConnectionFrame::GetPing(GetPingFrame {})).await;

        let start = Instant::now();
        match self.read_frame().await? {
            ConnectionFrame::PingResponse(_) => {},
            _ => {
                return Err("Wrong frame received!".to_string());
            },
        };
        Ok(Instant::now().duration_since(start).as_micros())
    }

    pub async fn retrieve_info(&mut self) -> Result<(), String> {
        if self.state != ConnectionState::Connected {
            return Err("Failed to retrieve info, connection is not in connected state!".to_string());
        }

        self.write_frame(ConnectionFrame::GetInfo(GetInfoFrame {})).await;

        let info_response = match self.read_frame().await? {
            ConnectionFrame::InfoResponse(frame) => frame,
            _ => {
                return Err("Wrong frame received!".to_string());
            }
        };

        let ping = self.get_ping().await?;

        self.state = ConnectionState::InfoRetrieved;
        self.info = Some(ConnectionInfo {
            ping: ping as i64,
            file_ids: info_response.file_ids,
            known_peers: info_response.known_peers,
        });
        Ok(())
    }
    
    // todo: currently there is a problem, that when request frame is sent through the open connection
    // the response frame from other request may appear in between. The solution is to create a task with 
    // channel to return the result to
    pub async fn get_file_piece(&mut self, file_id: String, piece: u64) -> Result<FilePieceResponseFrame, String> {
        self.write_frame(ConnectionFrame::GetFilePiece(GetFilePieceFrame { file_id, piece })).await;

        loop {
            let res: Option<FilePieceResponseFrame> = match self.read_frame().await? {
                ConnectionFrame::FilePieceResponse(r) => Some(r),
                f => {
                    println!("Wrong frame received: {:?}", f);
                    None
                },
            };
            if let Some(r) = res {
                return Ok(r)
            }
        }
    }
    
    pub async fn send_file_piece_download_status(&mut self, file_id: String, piece: u64, status: PieceDownloadStatus) {
        self.write_frame(ConnectionFrame::FilePieceDownloadStatusResponse(FilePieceDownloadStatusResponseFrame {
            file_id,
            piece,
            status,
        })).await;
    }
}
