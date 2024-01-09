use eyre::{eyre, Error, Result};

use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvmBlockHeaderFromRpc {
    pub number: String,
    pub hash: String,
    pub difficulty: String,
    pub extra_data: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub logs_bloom: String,
    pub miner: String,
    pub mix_hash: String,
    pub nonce: String,
    pub parent_hash: String,
    pub receipts_root: String,
    pub sha3_uncles: String,
    pub size: String,
    pub state_root: String,
    pub timestamp: String,
    pub total_difficulty: String,
    pub transactions_root: String,
    pub base_fee_per_gas: Option<String>,
    pub withdrawals_root: Option<String>,
}

pub struct Fetcher {
    pub client: Arc<Client>,
    pub url: String,
}

impl Fetcher {
    pub fn new(url: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            url,
        }
    }

    pub async fn fetch_block_by_number(&self, block_hash: &str) -> Result<EvmBlockHeaderFromRpc> {
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByHash",
            "params": [block_hash, false],
            "id": 1,
        });

        let rpc_response = self
            .client
            .post(&self.url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&rpc_request)
            .send()
            .await?;

        if rpc_response.status() != 200 {
            return Err(eyre!(
                "Request failed with status code {}",
                rpc_response.status()
            ));
        }

        let body: Value = rpc_response.json().await?;

        let evm_block_header_from_rpc_result: EvmBlockHeaderFromRpc = body
            .get("result")
            .ok_or_else(|| eyre!("Result field not found"))
            .and_then(|result| {
                from_value::<EvmBlockHeaderFromRpc>(result.clone()).map_err(Error::msg)
            })?;

        Ok(evm_block_header_from_rpc_result)
    }
}
