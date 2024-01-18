pub mod command_parser;
pub mod protocol_helpers;
pub mod utils;

use base64::Engine;
use command_parser::ServerArgs;
use protocol_helpers::{recv_loop, recv_u64};

use nix::sys::socket::listen as listen_vsock;
use nix::sys::socket::{accept, bind, socket};
use nix::sys::socket::{AddressFamily, SockAddr, SockFlag, SockType};
use types::DecryptionRequest;

const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;
const BUF_MAX_LEN: usize = 8192;
// Maximum number of outstanding connections in the socket's listen queue
const BACKLOG: usize = 128;

// Accept connections on a certain port and print the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let socket_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(|err| format!("Create socket failed: {:?}", err))
    .unwrap();

    let sockaddr = SockAddr::new_vsock(VMADDR_CID_ANY, args.port);

    bind(socket_fd, &sockaddr)
        .map_err(|err| format!("Bind failed: {:?}", err))
        .unwrap();

    listen_vsock(socket_fd, BACKLOG)
        .map_err(|err| format!("Listen failed: {:?}", err))
        .unwrap();

    loop {
        let fd = accept(socket_fd)
            .map_err(|err| format!("Accept failed: {:?}", err))
            .unwrap();

        // Receive data from vsock
        let len = recv_u64(fd).unwrap();
        let mut buf = [0u8; BUF_MAX_LEN];
        recv_loop(fd, &mut buf, len).unwrap();

        // slice the buffer to the actual length of the received data
        let data = String::from_utf8(buf.to_vec()).unwrap().as_str()[..len as usize].to_string();

        let decryption_request: DecryptionRequest = serde_json::from_str(&data).unwrap();

        let decrypted = decrypt(decryption_request);
        println!("Decrypted: {}", decrypted);
    }
}

fn decrypt(request: DecryptionRequest) -> String {
    // call call_executable with path 'kmstool_enclave_cli'
    let path = "./kmstool_enclave_cli";
    let args = vec![
        "decrypt",
        "--region",
        &request.region,
        "--proxy-port",
        &request.proxy_port,
        "--aws-access-key-id",
        &request.access_key_id,
        "--aws-secret-access-key",
        &request.secret_access_key,
        "--aws-session-token",
        &request.session_token,
        "--ciphertext",
        &request.ciphertext,
        "--key-id",
        &request.key_id,
        "--encryption-algorithm",
        &request.encryption_algorithm,
    ];
    let output: String = call_executable(path, &args).unwrap();
    println!("Output: {} ({:?})", output, output.as_bytes());
    String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(output)
            .unwrap(),
    )
    .unwrap()
}

// function that calls executable files with supplied path and arguments, return the stdout
fn call_executable(path: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(path)
        .args(args)
        .output()
        .map_err(|err| format!("Failed to execute command: {:?}", err))
        .unwrap();

    println!("{:?}", output);

    if !output.status.success() {
        return Err(format!(
            "Command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    let output = String::from_utf8(output.stdout)
        .map_err(|err| format!("The command's stdout is not UTF-8: {:?}", err.utf8_error()))
        .unwrap();

    Ok(String::from(
        output.split("PLAINTEXT: ").collect::<Vec<&str>>()[1]
            .split("\n")
            .collect::<Vec<&str>>()[0],
    ))
}
