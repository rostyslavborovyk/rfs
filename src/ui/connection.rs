use std::io::{Read, Write};
use std::net::TcpStream;
use serde_cbor::{from_slice, to_vec};
use crate::peer::connection::{ConnectionFrame, ConnectionInfo, GetInfoFrame};
use crate::peer::enums::ConnectionState;
use crate::values::DEFAULT_BUFFER_SIZE;

pub struct Connection {
    stream: TcpStream,
    state: ConnectionState,
    pub info: Option<ConnectionInfo>,
    buffer: [u8; DEFAULT_BUFFER_SIZE],
}

impl Connection {
    pub fn from_address(address: &String) -> Option<Self> {
        match TcpStream::connect(address) {
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

    pub fn read_frame(&mut self) -> Result<ConnectionFrame, String> {
        let n_bytes = match self.stream.read(&mut self.buffer) {
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

    pub fn write_frame(&mut self, frame: ConnectionFrame) {
        let data = to_vec(&frame).expect("Failed to serialize GetInfo frame!");
        println!("Writing frame with size {}", data.len());
        self.stream.write_all(data.as_ref()).expect("Failed to send GetInfo frame to the peer");
    }

    pub fn retrieve_info(&mut self) -> Result<ConnectionInfo, String> {
        self.write_frame(ConnectionFrame::GetInfo(GetInfoFrame {}));

        let info_response = match self.read_frame()? {
            ConnectionFrame::InfoResponse(frame) => frame,
            _ => {
                return Err("Wrong frame received!".to_string());
            }
        };

        Ok(ConnectionInfo {
            ping: 0,
            file_ids: info_response.file_ids,
            known_peers: info_response.known_peers,
        })
    }
}
