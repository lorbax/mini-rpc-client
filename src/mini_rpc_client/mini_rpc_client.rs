// TODO
//  - manage id in RpcResult messages
use base64::Engine;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::Bytes,
    header::{AUTHORIZATION, CONTENT_TYPE},
    Request,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::BlockHash;

#[derive(Clone, Debug)]
pub struct MiniRpcClient {
    client: Client<HttpConnector, Full<Bytes>>,
    url: String,
    auth: Auth,
}

impl MiniRpcClient {
    pub fn new(url: String, auth: Auth) -> MiniRpcClient {
        let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build_http();
        MiniRpcClient { client, url, auth }
    }
    
    pub async fn get_mempool_entry(
        &self,
        txid: &String,
    ) -> Result<GetMempoolEntryResult, RpcError> {
        // mempool inclusion is hardcoded to true, set it optional for the caller
        let response = self
            .send_json_rpc_request("getmempoolentry", json!([txid]))
            .await;
        match response {
            Ok(result_hex) => {
                let result_deserialized: JsonRpcResult<GetMempoolEntryResult> =
                    serde_json::from_str(&result_hex).map_err(|e| {
                        RpcError::Deserialization(e.to_string()) // TODO manage message ids
                    })?;
                if let Some(mempool_entry) = result_deserialized.result {
                    Ok(mempool_entry)
                } else if let Some(error) = result_deserialized.error {
                    return Err(RpcError::JsonRpc(error));
                } else {
                    return Err(RpcError::ResultErrorBothNone);
                }
            }
            Err(error) => {
                dbg!(&error);
                Err(error)
            }
        }
    }

    pub async fn get_tx_output(
        &self,
        txid: &String,
        vout_number: u32,
    ) -> Result<GetTxOutResult, RpcError> {
        // mempool inclusion is hardcoded to true, set it optional for the caller
        let response = self
            .send_json_rpc_request("gettxout", json!([txid, vout_number, true]))
            .await;
        match response {
            Ok(result_hex) => {
                let result_deserialized: JsonRpcResult<GetTxOutResult> =
                    serde_json::from_str(&result_hex).map_err(|e| {
                        RpcError::Deserialization(e.to_string()) // TODO manage message ids
                    })?;
                if let Some(txout) = result_deserialized.result {
                    Ok(txout)
                } else if let Some(error) = result_deserialized.error {
                    return Err(RpcError::JsonRpc(error));
                } else {
                    return Err(RpcError::ResultErrorBothNone);
                }
            }
            Err(error) => {
                dbg!(&error);
                Err(error)
            }
        }
    }

    // HOW TO DECODE A TRANSACTION:
    // use bitcoin::{consensus::encode::deserialize as consensus_decode, Transaction};
    // use hex::decode;
    //     // result: GetRawTransactionVerboseResult
    //     let transaction_hex: String = result.hex.unwrap();
    //     let transaction_bytes = decode(transaction_hex).expect("Decoding failed");
    //     let transaction: Transaction = consensus_decode(&transaction_bytes).expect("Deserialization failed");
    pub async fn get_raw_transaction(
        &self,
        txid: &String,
        block_hash: Option<&BlockHash>,
        verbose: bool,
    ) -> Result<GetRawTransactionVerboseResult, RpcError> {
        let response = match block_hash {
            Some(hash) => {
                self.send_json_rpc_request("getrawtransaction", json!([txid, verbose, hash]))
            }
            None => self.send_json_rpc_request("getrawtransaction", json!([txid, verbose])),
        }
        .await;
        match response {
            Ok(result_hex) => {
                let result_deserialized: JsonRpcResult<GetRawTransactionVerboseResult> = serde_json::from_str(&result_hex)
                    .map_err(|e| {
                        RpcError::Deserialization(e.to_string()) // TODO manage message ids
                    })?;
                result_deserialized.result.ok_or_else(|| RpcError::Other("Result not found".to_string()))
            }
            Err(error) => Err(error),
        }
    }

    //pub async fn get_raw_transaction(
    //    &self,
    //    txid: &String,
    //    block_hash: Option<&BlockHash>,
    //) -> Result<Transaction, RpcError> {
    //    let response = match block_hash {
    //        Some(hash) => {
    //            self.send_json_rpc_request("getrawtransaction", json!([txid, false, hash]))
    //        }
    //        None => self.send_json_rpc_request("getrawtransaction", json!([txid, false])),
    //    }
    //    .await;
    //    match response {
    //        Ok(result_hex) => {
    //            let result_deserialized: JsonRpcResult<String> = serde_json::from_str(&result_hex)
    //                .map_err(|e| {
    //                    RpcError::Deserialization(e.to_string()) // TODO manage message ids
    //                })?;
    //            let transaction_hex: String = if let Some(hex) = result_deserialized.result {
    //                hex
    //            } else {
    //                match result_deserialized.error {
    //                    Some(error) => return Err(RpcError::JsonRpc(error)),
    //                    None => {
    //                        return Err(RpcError::ResultErrorBothNone);
    //                    }
    //                }
    //            };
    //            let transaction_bytes = decode(transaction_hex).expect("Decoding failed");
    //            Ok(consensus_decode(&transaction_bytes).expect("Deserialization failed"))
    //        }
    //        Err(error) => Err(error),
    //    }
    //}

    pub async fn get_raw_mempool(&self) -> Result<Vec<String>, RpcError> {
        let response = self.send_json_rpc_request("getrawmempool", json!([])).await;
        match response {
            Ok(result_hex) => {
                let result_deserialized: JsonRpcResult<Vec<String>> =
                    serde_json::from_str(&result_hex).map_err(|e| {
                        RpcError::Deserialization(e.to_string()) // TODO manage message ids
                    })?;
                let mempool: Vec<String> = result_deserialized
                    .result
                    .ok_or_else(|| RpcError::Other("Result not found".to_string()))?;
                Ok(mempool)
            }
            Err(error) => Err(error),
        }
    }

    pub async fn submit_block(&self, block_hex: String) -> Result<(), RpcError> {
        let response = self
            .send_json_rpc_request("submitblock", json!([block_hex]))
            .await;

        match response {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }

    async fn send_json_rpc_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<String, RpcError> {
        let client = &self.client;
        let (username, password) = self.auth.clone().get_user_pass();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1, //TODO manage message ids
        };

        let request_body = match serde_json::to_string(&request) {
            Ok(body) => body,
            Err(e) => return Err(RpcError::Serialization(e.to_string())),
        };

        let req = Request::builder()
            .method("POST")
            .uri(self.url.as_str())
            .header(CONTENT_TYPE, "application/json")
            .header(
                AUTHORIZATION,
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD
                        .encode(format!("{}:{}", username, password))
                ),
            )
            .body(Full::<Bytes>::from(request_body))
            .map_err(|e| RpcError::Http(e.to_string()))?;

        let response = client
            .request(req)
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;

        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?
            .to_bytes()
            .to_vec();

        if status.is_success() {
            String::from_utf8(body).map_err(|e| {
                RpcError::Deserialization(e.to_string()) // TODO manage message ids
            })
        } else {
            return Err(RpcError::Http("Http reply with error status".to_string()));
        }
    }
}

#[derive(Clone, Debug)]
pub struct Username(pub String);
#[derive(Clone, Debug)]
pub struct Password(pub String);

#[derive(Clone, Debug)]
pub struct Auth {
    username: Username,
    password: Password,
}

impl Auth {
    pub fn get_user_pass(self) -> (String, String) {
        (self.username.0, self.password.0)
    }
    pub fn new(username: Username, password: Password) -> Auth {
        Auth { username, password }
    }
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResult<T> {
    result: Option<T>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub enum RpcError {
    // TODO this type is slightly incorrect, as the JsonRpcError evaluates a generic that is meant
    // for the result field of JsonRpcResult struct. This should be corrected
    JsonRpc(JsonRpcError),
    Deserialization(String),
    Serialization(String),
    Http(String),
    // returned if a message with both Result and Error fields are empty
    ResultErrorBothNone,
    Other(String),
}

impl TryFrom<JsonRpcResult<String>> for RpcError {
    type Error = ();
    fn try_from(result: JsonRpcResult<String>) -> Result<Self, ()> {
        match result.error {
            Some(error) => Ok(RpcError::JsonRpc(error)),
            None => Err(()),
        }
    }
}

// STRUCTURES USED FOR PARSING THE JSON RPC RESPONSE

#[derive(Deserialize, Debug)]
pub struct GetRawTransactionVerboseResult {
    #[serde(rename = "in_active_chain")]
    in_active_chain: Option<bool>,
    pub hex: Option<String>,
    txid: String,
    hash: String,
    version: i32,
    size: u32,
    vsize: u32,
    weight: u32,
    locktime: u32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    blockhash: Option<String>,
    confirmations: Option<u32>,
    time: Option<u64>,
    blocktime: Option<u64>,
}
#[derive(Deserialize, Debug)]
struct Vin {
    txid: Option<String>,
    vout: Option<u32>,
    scriptSig: Option<ScriptSig>,
    sequence: u64,
    coinbase: Option<String>,
    txinwitness: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct ScriptSig {
    asm: String,
    hex: String,
}

#[derive(Deserialize, Debug)]
struct Vout {
    value: f64,
    n: u32,
    scriptPubKey: ScriptPubKey,
}

#[derive(Deserialize, Debug)]
pub struct GetTxOutResult {
     bestblock: String,
     confirmations: Option<u64>,
     value: f64,
    #[serde(rename = "scriptPubKey")]
     script_pub_key: ScriptPubKey,
     coinbase: bool,
}

#[derive(Deserialize, Debug)]
pub struct ScriptPubKey {
     asm: String,
     hex: String,
     req_sigs: Option<u32>,
    #[serde(rename = "type")]
     script_type: String,
     addresses: Option<Vec<String>>,
}


#[derive(Deserialize, Debug)]
pub struct GetMempoolEntryResult {
     vsize: u32,
     weight: u32,
     time: u64,
     height: u64,
     descendantcount: u32,
     descendantsize: u32,
     ancestorcount: u32,
     ancestorsize: u32,
     wtxid: String,
     fees: Fees,
     depends: Vec<String>,
     spentby: Vec<String>,
    #[serde(rename = "bip125-replaceable")]
     bip125_replaceable: bool,
     unbroadcast: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Fees {
     base: f64,
     modified: f64,
     ancestor: f64,
     descendant: f64,
}
