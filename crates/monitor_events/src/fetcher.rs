use eyre::{eyre, Error, OptionExt, Result};

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


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptimismOutputAtBlock {
    pub version: String,
    pub output_root: String,
    pub block_ref: BlockRef,
    pub withdrawal_storage_root: String,
    pub state_root: String,
    pub sync_status: SyncStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockRef {
    pub hash: String,
    pub number: u64,
    pub parent_hash: String,
    pub timestamp: u64,
    pub l1origin: L1Origin,
    pub sequence_number: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L1Origin {
    pub hash: String,
    pub number: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub current_l1: L1Block,
    pub current_l1_finalized: L1Block,
    pub head_l1: L1Block,
    pub safe_l1: L1Block,
    pub finalized_l1: L1Block,
    pub unsafe_l2: L2Block,
    pub safe_l2: L2Block,
    pub finalized_l2: L2Block,
    pub pending_safe_l2: L2Block,
    pub cross_unsafe_l2: L2Block,
    pub local_safe_l2: L2Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L1Block {
    pub hash: String,
    pub number: u64,
    pub parent_hash: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L2Block {
    pub hash: String,
    pub number: u64,
    pub parent_hash: String,
    pub timestamp: u64,
    pub l1origin: L1Origin,
    pub sequence_number: u64,
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

    pub async fn fetch_block_by_hash(&self, block_hash: &str) -> Result<EvmBlockHeaderFromRpc> {
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
            .ok_or_eyre("Result field not found")
            .and_then(|result| {
                from_value::<EvmBlockHeaderFromRpc>(result.clone()).map_err(Error::msg)
            })?;

        Ok(evm_block_header_from_rpc_result)
    }

    pub async fn fetch_optimism_output_at_block(&self, block_number: &str) -> Result<OptimismOutputAtBlock> {
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": "optimism_outputAtBlock",
            "params": [block_number],
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

        let optimism_output_at_block_from_rpc_result: OptimismOutputAtBlock = body
            .get("result")
            .ok_or_eyre("Result field not found")
            .and_then(|result| {
                from_value::<OptimismOutputAtBlock>(result.clone()).map_err(Error::msg)
            })?;

        Ok(optimism_output_at_block_from_rpc_result)
    }
}
