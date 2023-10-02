use dotenv::dotenv;
use ethers::prelude::*;
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
    let RPC_URL: &str = &std::env::var("RPC_URL").expect("RPC_URL must be set.");
    let provider = Provider::<Http>::try_from(RPC_URL)?;
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
        let l2_output_index = U256::from_big_endian(&log.topics[2].as_bytes());
        let l2_block_number = U256::from_big_endian(&log.topics[3].as_bytes());
        let l1_timestamp = U256::from_big_endian(&log.data[29..32]);

        println!(
            "output_root = {output_root}, l2OutputIndex = {l2_output_index}, l2BlockNumber = {l2_block_number}, l1Timestamp = {l1_timestamp}",
        );

        // We can get it from Event
        // if let Ok(output_proposal) = contract.get_l2_output(l2OutputIndex).call().await {
        //     println!("output_proposal is {output_proposal:?}");
        // }
    }
    listen_output_proposed_events(&contract).await?;
    Ok(())
}

async fn listen_output_proposed_events(contract: &IPROXY<Provider<Http>>) -> Result<()> {
    let events = contract.event::<OutputProposed>().from_block(0);
    let mut stream = events.stream().await?;
    println!("Started monitoring. Will response when event is detacted");
    loop {
        match stream.next().await {
            Some(result) => match result {
                Ok(output_proposed) => {
                    let output_root = output_proposed.output_root;
                    let l2_output_index = output_proposed.l1_output_index;
                    let l2_block_number = output_proposed.l2_block_number;
                    let l1_time_stamp = output_proposed.l1_time_stamp;
                    println!(
                            "output_root = {output_root}, l2OutputIndex = {l2_output_index}, l2BlockNumber = {l2_block_number}, l1Timestamp = {l1_time_stamp}",
                        );
                }
                Err(contract_error) => println!("contract error: {contract_error:?}"),
            },
            _ => {}
        }
    }
}
