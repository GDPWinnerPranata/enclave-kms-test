use clap::{App, AppSettings, Arg};

use vsock_server::command_parser::ServerArgs;
use vsock_server::create_app;
use vsock_server::server;

fn main() {
    let app = create_app!();
    let args = app.get_matches();

    let server_args = ServerArgs::new_with(&args).unwrap();
    server(server_args).unwrap();
}
