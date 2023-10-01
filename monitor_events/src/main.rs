use dotenv::dotenv;
use ethers::prelude::*;
use ethers_contract_derive::EthAbiType;
use ethers_core::abi::{AbiType, ParamType};
use ethers_core::types::*;
use eyre::Result;
use std::sync::Arc;

#[derive(EthEvent, Debug)]
#[ethevent(abi = "OutputProposed(bytes32,uint256,uint256,uint256)")]
struct OutputProposed {
    #[ethevent(indexed, name = "outputRoot")]
    output_root: Bytes,
    #[ethevent(indexed, name = "l2OutputIndex")]
    l1_output_index: I256,
    #[ethevent(indexed, name = "l2BlockNumber")]
    l2_block_number: I256,
    l1_time_stamp: I256,
}
abigen!(IPROXY, "./l1outputoracle.json");

const OP_PROPOSER_ADDRESS: &str = "0xdfe97868233d1aa22e815a266982f2cf17685a27";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let HTTP_URL: &str = &std::env::var("HTTP_URL").expect("MAILCOACH_API_TOKEN must be set.");
    let provider = Provider::<Http>::try_from(HTTP_URL)?;
    let client = Arc::new(provider);
    let address: Address = OP_PROPOSER_ADDRESS.parse()?;

    let contract = IPROXY::new(address, client.clone());
    let filter = Filter::new()
        .address(address)
        .event("OutputProposed(bytes32,uint256,uint256,uint256)")
        .from_block(0);

    let logs = client.get_logs(&filter).await?;
    println!("{} pools found!", logs.iter().len());
    for log in logs.iter() {
        let output_root = Bytes::from(log.topics[1].as_bytes().to_vec());
        let l2OutputIndex = U256::from_big_endian(&log.topics[2].as_bytes());
        let l2BlockNumber = U256::from_big_endian(&log.topics[3].as_bytes());
        let l1Timestamp = U256::from_big_endian(&log.data[29..32]);

        println!(
            "output_root = {output_root}, l2OutputIndex = {l2OutputIndex}, l2BlockNumber = {l2BlockNumber}, l1Timestamp = {l1Timestamp}",
        );

        // We can get it from Event
        // if let Ok(output_proposal) = contract.get_l2_output(l2OutputIndex).call().await {
        //     println!("output_proposal is {output_proposal:?}");
        // }
    }
    Ok(())
}
