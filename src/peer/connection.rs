use serde::{Deserialize, Serialize};
use serde_json::{Error, from_slice, to_vec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Instant;
use crate::peer::enums::ConnectionState;


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
    buffer: [u8; 1024],
}

impl Connection {
    pub async fn from_address(address: &String) -> Option<Self> {
        match TcpStream::connect(address).await {
            Ok(stream) => Some(
                Connection {
                    stream,
                    state: ConnectionState::Connected,
                    buffer: [0; 1024],
                    info: None,
                }
            ),
            Err(_) => None,
        }
    }

    pub async fn from_stream(stream: TcpStream) -> Self {
        Connection {
            stream,
            state: ConnectionState::Connected,
            buffer: [0; 1024],
            info: None,
        }
    }

    fn get_frame(&self, buffer: &[u8]) -> Result<ConnectionFrame, Error> {
        let frame_result: Result<ConnectionFrame, _> = from_slice(buffer);
        frame_result
    }

    pub async fn read_frame(&mut self) -> Option<ConnectionFrame> {
        let n_bytes = match self.stream.read(&mut self.buffer).await {
            Ok(0) => {
                eprintln!("No bytes received from connection, closing");
                return None;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return None;
            }
        };

        let bytes = &self.buffer[..n_bytes];

        let frame = match self.get_frame(bytes) {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return None;
            }
        };
        Some(frame)
    }

    pub async fn write_frame(&mut self, frame: ConnectionFrame) {
        let data = to_vec(&frame).expect("Failed to serialize GetInfo frame!");
        self.stream.write_all(data.as_ref()).await.expect("Failed to send GetInfo frame to the peer");
    }

    pub async fn retrieve_info(&mut self) -> Result<(), String> {
        if self.state != ConnectionState::Connected {
            return Err("Failed to retrieve info, connection is not in connected state!".to_string());
        }

        self.write_frame(ConnectionFrame::GetInfo(GetInfoFrame {})).await;

        let info_response = match self.read_frame().await.ok_or("Invalid data received!")? {
            ConnectionFrame::InfoResponse(frame) => frame,
            _ => {
                return Err("Wrong frame received!".to_string());
            }
        };

        self.write_frame(ConnectionFrame::GetPing(GetPingFrame {})).await;

        let start = Instant::now();
        match self.read_frame().await {
            Some(ConnectionFrame::PingResponse(_)) => {},
            Some(_) => {
                return Err("Wrong frame received!".to_string());
            },
            None => {
                return Err("Invalid data received!".to_string());
            }
        };
        let ping = Instant::now().duration_since(start).as_micros();

        self.state = ConnectionState::InfoRetrieved;
        self.info = Some(ConnectionInfo {
            ping,
            file_ids: info_response.file_ids,
        });
        Ok(())
    }
}
