use std::str::FromStr;

use base64::{Engine as _, engine::general_purpose};

use serde_json::json;

use embedded_svc::http::client::Client;
use embedded_svc::http::Headers;
use esp_idf_svc::http::{
    client::{Configuration, EspHttpConnection},
    Method,
};
use solana_transaction::{Hash, Transaction};

const RPC_URL: &str = "https://api.devnet.solana.com";

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum SolanaRpcMethod {
    GetLatestBlockhash,
    GetBalance(String),
    GetTransaction(String),
    GetAccountInfo(String),
    GetProgramAccounts(String),
    GetRecentBlockhash,
    GetSlot,
    GetVersion,
    SendTransaction(String),
}

pub fn get_latest_blockhash() -> Result<Hash, String> {
    let result = sol_rpc_call(SolanaRpcMethod::GetLatestBlockhash)?;

    let blockhash_str = result["value"]["blockhash"]
        .as_str()
        .ok_or("No blockhash in response")?;

    Hash::from_str(blockhash_str).map_err(|e| format!("Hash parse: {:?}", e))
}

pub fn send_transaction(transaction: &Transaction) -> Result<String, String> {
    let transaction_bytes = bincode::serialize(transaction)
        .map_err(|e| format!("Transaction serialization failed: {:?}", e))?;

    let base64_transaction = general_purpose::STANDARD.encode(&transaction_bytes);

    send_transaction_base64(base64_transaction)
}

pub fn send_transaction_base64(base64_transaction: String) -> Result<String, String> {
    let result = sol_rpc_call(SolanaRpcMethod::SendTransaction(base64_transaction))?;

    let signature = result.as_str()
        .ok_or("Invalid response format: expected transaction signature")?
        .to_string();

    Ok(signature)
}

pub fn create_solana_payload(method: SolanaRpcMethod) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method.method_name(),
        "params": method.params()
    })
}

pub fn sol_rpc_call(method: SolanaRpcMethod) -> Result<serde_json::Value, String> {
    let connection = EspHttpConnection::new(&Configuration {
        timeout: Some(std::time::Duration::from_secs(30)),
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })
    .map_err(|e| format!("HTTP init: {:?}", e))?;

    let mut client = Client::wrap(connection);
    let payload = create_solana_payload(method);

    let payload_str =
        serde_json::to_string(&payload).map_err(|e| format!("JSON serialize: {:?}", e))?;

    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Length", &payload_str.len().to_string()),
    ];

    let mut request = client
        .request(Method::Post, RPC_URL, &headers)
        .map_err(|e| format!("Request: {:?}", e))?;

    request
        .write(payload_str.as_bytes())
        .map_err(|e| format!("Write: {:?}", e))?;

    let response = request
        .submit()
        .map_err(|e| format!("Submit: {:?}", e))?;

    let status = response.status();
    if !(200..=299).contains(&status) {
        return Err(format!("HTTP Error: Status code {}", status));
    }

    let mut response_body = Vec::with_capacity(response.content_len().unwrap_or(0) as usize);
    let mut reader = response;
    let mut buf = [0u8; 256];
    loop {
        let size = reader.read(&mut buf).map_err(|e| format!("Read: {:?}", e))?;
        if size == 0 {
            break;
        }
        response_body.extend_from_slice(&buf[..size]);
    }
    let response_str = str::from_utf8(&response_body).map_err(|e| format!("UTF-8: {:?}", e))?;
    let json_response: serde_json::Value = serde_json::from_str(response_str).map_err(|e| format!("JSON parse: {:?}", e))?;

    Ok(json_response["result"].clone())
}

impl SolanaRpcMethod {
    pub fn method_name(&self) -> &'static str {
        match self {
            SolanaRpcMethod::GetLatestBlockhash => "getLatestBlockhash",
            SolanaRpcMethod::GetBalance(_) => "getBalance",
            SolanaRpcMethod::GetTransaction(_) => "getTransaction",
            SolanaRpcMethod::GetAccountInfo(_) => "getAccountInfo",
            SolanaRpcMethod::GetProgramAccounts(_) => "getProgramAccounts",
            SolanaRpcMethod::GetRecentBlockhash => "getRecentBlockhash",
            SolanaRpcMethod::GetSlot => "getSlot",
            SolanaRpcMethod::GetVersion => "getVersion",
            SolanaRpcMethod::SendTransaction(_) => "sendTransaction",
        }
    }

    pub fn params(&self) -> serde_json::Value {
        match self {
            SolanaRpcMethod::GetLatestBlockhash => {
                json!([{"commitment": "confirmed"}])
            }
            SolanaRpcMethod::GetBalance(wallet) => {
                json!([wallet])
            }
            SolanaRpcMethod::GetTransaction(signature) => {
                json!([signature, {"encoding": "jsonParsed"}])
            }
            SolanaRpcMethod::GetAccountInfo(account) => {
                json!([account, {"encoding": "base64"}])
            }
            SolanaRpcMethod::GetProgramAccounts(program) => {
                json!([program, {"encoding": "base64"}])
            }
            SolanaRpcMethod::GetRecentBlockhash => {
                json!([])
            }
            SolanaRpcMethod::GetSlot => {
                json!([])
            }
            SolanaRpcMethod::GetVersion => {
                json!([])
            }
            SolanaRpcMethod::SendTransaction(transaction) => {
                json!([transaction, {
                    "encoding": "base64",
                    "skipPreflight": false,
                    "preflightCommitment": "confirmed",
                    "maxRetries": 3
                }])
            }
        }
    }
}