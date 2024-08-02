use distributed_fs::peer::connection::{ConnectionFrame, GetFileFrame};
use distributed_fs::ui::connection::Connection;
use distributed_fs::values::LOCAL_PEER_ADDRESS;

fn main() {
    let frame = ConnectionFrame::GetFile(GetFileFrame {file_id: "4148f04f-41e3-4f39-94e8-155bc6dcd3ae".to_string()});
    let mut connection = Connection::from_address(&LOCAL_PEER_ADDRESS.to_string()).unwrap();
    connection.write_frame(frame);
}
