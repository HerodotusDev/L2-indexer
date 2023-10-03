use dotenv::dotenv;
use ethers::prelude::*;
use eyre::Result;
use std::{
    sync::Arc,
    thread,
    time::{self, Duration},
};

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
const BLOCK_DELAY: U64 = U64([20]);
const POLL_PERIOD: Duration = time::Duration::from_secs(60);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let rpc_url: &str = &std::env::var("RPC_URL").expect("RPC_URL must be set.");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);
    let address: Address = OP_PROPOSER_ADDRESS.parse()?;

    let mut from_block_num = U64([0]);
    let mut new_block_num = client.get_block_number().await? - BLOCK_DELAY;

    let mut filter = Filter::new()
        .address(address)
        .event("OutputProposed(bytes32,uint256,uint256,uint256)")
        .from_block(0)
        .to_block(new_block_num - 1);

    loop {
        let logs = client.get_logs(&filter).await?;
        println!(
            "from {from_block_num:?} to {new_block_num:?}, {} pools found!",
            logs.iter().len()
        );

        for log in logs.iter() {
            let output_root = Bytes::from(log.topics[1].as_bytes().to_vec());
            let l2_output_index = U256::from_big_endian(&log.topics[2].as_bytes());
            let l2_block_number = U256::from_big_endian(&log.topics[3].as_bytes());
            let l1_timestamp = U256::from_big_endian(&log.data[29..32]);

            println!(
                "output_root = {output_root}, l2OutputIndex = {l2_output_index}, l2BlockNumber = {l2_block_number}, l1Timestamp = {l1_timestamp}",
            );
        }
        thread::sleep(POLL_PERIOD);

        from_block_num = new_block_num;
        new_block_num = client.get_block_number().await? - BLOCK_DELAY;
        filter = filter
            .from_block(from_block_num)
            .to_block(new_block_num - 1);
    }
}
