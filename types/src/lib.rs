use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptionRequest {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub region: String,
    pub ciphertext: String,
    pub key_id: String,
    pub encryption_algorithm: String,
    pub proxy_port: String,
}