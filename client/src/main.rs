use clap::{App, AppSettings, Arg};

use vsock_client::client;
use vsock_client::command_parser::ClientArgs;
use vsock_client::create_app;

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env").ok();

    let app = create_app!();
    let args = app.get_matches();

    let client_args = ClientArgs::new_with(&args).unwrap();
    client(client_args).await.unwrap();
}
