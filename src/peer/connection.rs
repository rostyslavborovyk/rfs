use std::time::Duration;
use bytes::BytesMut;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
// todo: cbor serialization still produces 31Kb size for the frame with 16Kb of contents. 
// Maybe check some other available formats, or write own binary protocol?
use serde_cbor::{from_slice, to_vec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant, sleep};
use crate::peer::enums::ConnectionStatus;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFileFrame {
    pub file_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilePieceResponseFrame {
    pub file_id: String,
    pub piece: u64,
    pub content: Vec<u8>,
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
    status: ConnectionStatus,
    pub info: Option<ConnectionInfo>,
    buffer: BytesMut,
}

impl Connection {
    pub async fn from_address(address: &String) -> Result<Self, String> {
        match TcpStream::connect(address).await {
            Ok(stream) => 
                Ok(Connection {
                    stream,
                    status: ConnectionStatus::Connected,
                    buffer: BytesMut::with_capacity(DEFAULT_BUFFER_SIZE),
                    info: None,
                })
            ,
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn from_addresses(addresses: Vec<String>) -> Vec<Result<Connection, String>> {
        join_all(addresses.iter().map(|addr| async move {
            Connection::from_address(&addr.clone()).await
        })).await
    }

    pub async fn from_stream(stream: TcpStream) -> Self {
        Connection {
            stream,
            status: ConnectionStatus::Connected,
            buffer: BytesMut::with_capacity(DEFAULT_BUFFER_SIZE),
            info: None,
        }
    }

    // todo: connection may return more bytes than buffer can load. To rewrite it with an ability
    // for the buffer to store loaded bytes and read from stream again.
    pub async fn read_frame(&mut self) -> Result<ConnectionFrame, String> {
        let n_bytes = match self.stream.read(&mut self.buffer).await {
            Ok(0) => {
                Err("No bytes received from connection, closing".to_string())
            }
            Ok(n) => Ok(n),
            Err(e) => {
                Err(format!("Failed to read from socket; err = {:?}", e).to_string())
            }
        }?;

        from_slice(&self.buffer[..n_bytes]).map_err(|err| format!("Error when parsing frame {err}"))
    }

    pub async fn write_frame(&mut self, frame: ConnectionFrame) {
        let data = to_vec(&frame).expect("Failed to serialize GetInfo frame!");
        println!("Writing frame with size {}", data.len());
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
        if self.status != ConnectionStatus::Connected {
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

        self.status = ConnectionStatus::InfoRetrieved;
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
        
        // todo: this sleep is required, because apparently, to quick write between sockets messes up the 
        // data. To resolve it later
        sleep(Duration::from_millis(10)).await;

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
}
