use std::sync::Arc;
use tokio::sync::Mutex;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::listener::{refresh_pings_for_peers, serve_listener};
use distributed_fs::peer::state::State;

use clap::Parser;
use distributed_fs::domain::config::FSConfig;
use distributed_fs::domain::fs::check_folders;
use distributed_fs::values::LOCAL_PEER_ADDRESS;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: Option<String>,

    #[arg(short, long)]
    rfs_dir: Option<String>,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    
    let fs_config = FSConfig::new(args.rfs_dir);
    check_folders(&fs_config);
    
    let sharable_state_container = Arc::new(Mutex::new(State::new(fs_config.clone())));

    let address = args.address.unwrap_or(LOCAL_PEER_ADDRESS.to_string());
    
    let mut client = Client::new(address.clone(), sharable_state_container.clone());
    

    
    client.load_state(address.clone(), &fs_config).await.unwrap();

    println!("Starting peer with address {} and fs location {} ...", address, fs_config.rfs_dir);
    
    let mut c = sharable_state_container.clone();
    tokio::spawn(async move {
        refresh_pings_for_peers(&mut c).await;
    });

    serve_listener(
        address,
        &mut sharable_state_container.clone(),
    ).await
}