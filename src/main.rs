mod mini_rpc_client;

use crate::mini_rpc_client::mini_rpc_client::{Auth, MiniRpcClient, Password, Username};
use bitcoin::{consensus::encode::deserialize as consensus_decode, Transaction};
use hex::decode;
use tokio;
const RPC_USER: &str = "user";
const RPC_PASSWORD: &str = "password";
const LOCAL_NODE: &str = "http://127.0.0.1:8332";

#[tokio::main]
async fn main() {
    let username = Username(RPC_USER.to_string());
    let password = Password(RPC_PASSWORD.to_string());
    let auth = Auth::new(username, password);
    let rpc_client = MiniRpcClient::new(LOCAL_NODE.to_string(), auth);
    let result = rpc_client
        .get_raw_transaction(
            &"2a7dde6523640a8f0d01c2100e9d13a165ee7cae6fa909c945cb0aa51591fed9".to_string(),
            None,
            true,
        )
        .await;
    dbg!(&result);
    let transaction_hex: String = result.unwrap().hex.unwrap();
    let transaction_bytes = decode(transaction_hex).expect("Decoding failed");
    let transaction: Transaction =
        consensus_decode(&transaction_bytes).expect("Deserialization failed");
    dbg!(transaction);

    println!("Hello, world!");
}
