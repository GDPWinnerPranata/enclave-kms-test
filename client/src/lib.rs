pub mod command_parser;
pub mod protocol_helpers;
pub mod utils;

use aws_config::BehaviorVersion;
use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::Client;
use aws_sdk_sts::operation::get_session_token::GetSessionTokenOutput;
use base64::Engine;
use command_parser::ClientArgs;
use protocol_helpers::{send_loop, send_u64};

use nix::sys::socket::{connect, shutdown, socket};
use nix::sys::socket::{AddressFamily, Shutdown, SockAddr, SockFlag, SockType};
use nix::unistd::close;
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, RawFd};
use types::DecryptionRequest;

const MAX_CONNECTION_ATTEMPTS: usize = 5;

#[derive(Debug)]
struct AWSServices {
    kms_client: Client,
    sts_session: GetSessionTokenOutput,
}

struct VsockSocket {
    socket_fd: RawFd,
}

impl VsockSocket {
    fn new(socket_fd: RawFd) -> Self {
        VsockSocket { socket_fd }
    }
}

impl Drop for VsockSocket {
    fn drop(&mut self) {
        shutdown(self.socket_fd, Shutdown::Both)
            .unwrap_or_else(|e| eprintln!("Failed to shut socket down: {:?}", e));
        close(self.socket_fd).unwrap_or_else(|e| eprintln!("Failed to close socket: {:?}", e));
    }
}

impl AsRawFd for VsockSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket_fd
    }
}

/// Initiate a connection on an AF_VSOCK socket
fn vsock_connect(cid: u32, port: u32) -> Result<VsockSocket, String> {
    let sockaddr = SockAddr::new_vsock(cid, port);
    let mut err_msg = String::new();

    for i in 0..MAX_CONNECTION_ATTEMPTS {
        let vsocket = VsockSocket::new(
            socket(
                AddressFamily::Vsock,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .map_err(|err| format!("Failed to create the socket: {:?}", err))
            .unwrap(),
        );
        match connect(vsocket.as_raw_fd(), &sockaddr) {
            Ok(_) => return Ok(vsocket),
            Err(e) => err_msg = format!("Failed to connect: {}", e),
        }

        // Exponentially backoff before retrying to connect to the socket
        std::thread::sleep(std::time::Duration::from_secs(1 << i));
    }

    Err(err_msg)
}

/// Send 'Hello, world!' to the server
pub async fn client(args: ClientArgs) -> Result<(), String> {
    let aws_services = get_aws_services().await;

    let vsocket = vsock_connect(args.cid, args.port).unwrap();
    let fd = vsocket.as_raw_fd();

    let data = "Hello, world!";
    let encrypted_data = encrypt(aws_services.kms_client, data).await.to_string();

    let credentials = aws_services.sts_session.credentials.unwrap();
    let payload = DecryptionRequest {
        access_key_id: credentials.access_key_id,
        secret_access_key: credentials.secret_access_key,
        session_token: credentials.session_token,
        region: dotenv::var("AWS_REGION").unwrap(),
        ciphertext: encrypted_data,
        key_id: dotenv::var("KEY_ID").unwrap(),
        encryption_algorithm: dotenv::var("ENCRYPTION_ALGORITHM").unwrap(),
        proxy_port: dotenv::var("PROXY_PORT").unwrap(),
    };
    let payload_serialized = serde_json::to_string::<DecryptionRequest>(&payload).unwrap();
    println!("Sent: {}", payload_serialized);

    let buf = payload_serialized.as_bytes();
    let len: u64 = buf.len().try_into().map_err(|err| format!("{:?}", err))?;
    println!("Length: {}", len);
    send_u64(fd, len).unwrap();
    send_loop(fd, buf, len).unwrap();

    Ok(())
}

async fn encrypt(kms_client: Client, text: &str) -> String {
    let key_id = dotenv::var("KEY_ID").unwrap();

    let blob = Blob::new(text.as_bytes());

    let resp = kms_client
        .encrypt()
        .key_id(key_id)
        .plaintext(blob)
        .send()
        .await
        .unwrap();
    let blob = resp.ciphertext_blob.unwrap();
    let bytes = blob.as_ref();

    base64::engine::general_purpose::STANDARD.encode(bytes)
}

async fn get_aws_services() -> AWSServices {
    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let sts_client = aws_sdk_sts::Client::new(&config);
    let session_token_output = sts_client.get_session_token().send().await.unwrap();

    AWSServices {
        kms_client: Client::new(&config),
        sts_session: session_token_output,
    }
}
