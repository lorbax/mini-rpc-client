mod mini_rpc_client;

use crate::mini_rpc_client::mini_rpc_client::{Auth, MiniRpcClient, Password, Username};
use bitcoin::{consensus::encode::deserialize as consensus_decode, Transaction};
use hex::decode;
use tokio;
const RPC_USER: &str = "user";
const RPC_PASSWORD: &str = "password";
const LOCAL_NODE: &str = "http://127.0.0.1:8332";
const VERBOSE: bool = false;
const BLOCKHASH: Option<&str> =
    Some("000000000000000000010d51283d100d342bcb99aed42ba5ad78cef84dd67b53");
//const BLOCKHASH: Option<&str> = None;
const TRANSACTION: &str = "c0666572ed187a8ca4340df82e287db5fc0bfd4c2ea5fbe1338ea567ce80ecb4";

#[tokio::main]
async fn main() {
    let username = Username(RPC_USER.to_string());
    let password = Password(RPC_PASSWORD.to_string());
    let auth = Auth::new(username, password);
    let rpc_client = MiniRpcClient::new(LOCAL_NODE.to_string(), auth);
    let transaction_hex: String = if VERBOSE {
        let result = rpc_client
            .get_raw_transaction_verbose(&TRANSACTION.to_string(), BLOCKHASH)
            .await;
        dbg!(&result);
        result.unwrap().hex.unwrap()
    } else {
        let result = rpc_client
            .get_raw_transaction(&TRANSACTION.to_string(), BLOCKHASH)
            .await;
        dbg!(&result);
        result.unwrap()
    };
    let transaction_bytes = decode(transaction_hex).expect("Decoding failed");
    let transaction: Transaction =
        consensus_decode(&transaction_bytes).expect("Deserialization failed");
    dbg!(transaction);

    println!("Hello, world!");
}
