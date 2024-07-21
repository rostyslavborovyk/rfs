use std::sync::Arc;
use tokio::sync::Mutex;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::listener::serve_listener;
use distributed_fs::peer::state_container::StateContainer;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8001"))]
    address: String,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    
    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));

    let mut client = Client::new(args.address.clone(), sharable_state_container.clone());
    
    client.load_metafiles().await.unwrap();

    serve_listener(
        args.address,
        &mut sharable_state_container.clone(),
    ).await
}