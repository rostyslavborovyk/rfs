use std::sync::Arc;
use clap::Parser;
use tokio::sync::Mutex;
use distributed_fs::domain::config::FSConfig;
use distributed_fs::domain::fs::check_folders;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::state::State;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Path to the file")]
struct Args {
    #[arg(short, long)]
    path: String,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    let fs_config = FSConfig::new(None);
    check_folders(&fs_config);
    let sharable_state_container = Arc::new(Mutex::new(State::new(fs_config.clone())));

    let client = Arc::new(Client::new("127.0.0.1:8000".to_string(), sharable_state_container.clone()));

    client.generate_meta_file(&args.path).await.unwrap();
    println!("Finished!")
}
