use serde_cbor::{from_slice, to_vec};
use distributed_fs::peer::connection::{ConnectionFrame, GetPingFrame};

fn main() {
    let frame = ConnectionFrame::GetPing(GetPingFrame {});
    let data = to_vec(&frame).expect("Failed to serialize GetInfo frame!");
    let new_frame: Result<ConnectionFrame, _> = from_slice(&data);
    println!("{:?}", new_frame)
}
