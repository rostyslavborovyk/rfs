use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant, sleep};
use crate::peer::enums::ConnectionState;
use crate::values::DEFAULT_BUFFER_SIZE;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetInfoFrame {}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponseFrame {
    pub file_ids: Vec<String>,
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

    #[serde(rename = "FilePieceResponse")]
    FilePieceResponse(FilePieceResponseFrame),
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub ping: u128,
    file_ids: Vec<String>,
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

    pub async fn from_stream(stream: TcpStream) -> Self {
        Connection {
            stream,
            state: ConnectionState::Connected,
            buffer: [0; DEFAULT_BUFFER_SIZE],
            info: None,
        }
    }

    fn get_frame(&self, buffer: &[u8]) -> Result<ConnectionFrame, String> {
        from_slice(buffer).map_err(|err| format!("Error when parsing frame {err}"))?
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

        let bytes = &self.buffer[..n_bytes];

        self.get_frame(bytes)
    }

    pub async fn write_frame(&mut self, frame: ConnectionFrame) {
        let data = to_vec(&frame).expect("Failed to serialize GetInfo frame!");
        println!("Writing frame with size {}", data.len());
        self.stream.write_all(data.as_ref()).await.expect("Failed to send GetInfo frame to the peer");
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

        self.write_frame(ConnectionFrame::GetPing(GetPingFrame {})).await;

        let start = Instant::now();
        match self.read_frame().await? {
            ConnectionFrame::PingResponse(_) => {},
            _ => {
                return Err("Wrong frame received!".to_string());
            },
        };
        let ping = Instant::now().duration_since(start).as_micros();

        self.state = ConnectionState::InfoRetrieved;
        self.info = Some(ConnectionInfo {
            ping,
            file_ids: info_response.file_ids,
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
