use crate::peer::connection::ConnectionInfo;
use crate::ui::connection::Connection;
use crate::values::LOCAL_PEER_ADDRESS;

pub fn get_info() -> ConnectionInfo {
    let mut connection = Connection::from_address(&LOCAL_PEER_ADDRESS.to_string()).unwrap();
    connection.retrieve_info().map_err(|err| {
        println!("Error when getting info {err}");
    }).unwrap()
}
