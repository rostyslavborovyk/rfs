use std::sync::Arc;
use clap::Parser;
use tokio::sync::Mutex;
use distributed_fs::peer::client::Client;
use distributed_fs::peer::state_container::StateContainer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Path to the file")]
struct Args {
    #[arg(short, long)]
    path: String,
}


#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    let sharable_state_container = Arc::new(Mutex::new(StateContainer::new()));

    let client = Arc::new(Client::new("127.0.0.1:8000".to_string(), sharable_state_container.clone()));

    client.generate_meta_file(&args.path).await.unwrap();
    println!("Finished!")
}
