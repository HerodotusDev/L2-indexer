use crate::{fetcher::Fetcher, ChainType};
use ethers::prelude::*;
use eyre::Result;

pub struct ArbitrumParameters {
    l2_output_root: Bytes,
    l2_block_hash: Bytes,
    l2_block_number: U256,
    l1_transaction_hash: Bytes,
    l1_block_number: U64,
    l1_transaction_index: U64,
    l1_block_hash: Bytes,
}

/// A function that creates a table if it doesn't exist, and returns the max block number in the table if it exists.
/// Parameters:
/// * table_name: The name of the postgres table
/// * client: The postgres client
/// Returns:
/// * Option<i32>: The max block number in the table if it exists, otherwise None
pub async fn create_arbitrum_table_if_not_exists(
    table_name: String,
    client: &tokio_postgres::Client,
) -> Result<Option<i32>, tokio_postgres::Error> {
    let create_table_query = format!("SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '{}') AS table_existence;", table_name);
    let rows = client.query(&create_table_query, &[]).await?;

    // And then check that we got back the same string we sent over.
    let exist: bool = rows[0].get(0);
    println!("Table exist : {exist}");
    if exist {
        let create_table_query = format!(
            "SELECT MAX(l1_block_number) as MaxBlock from {}",
            table_name
        );
        let rows = client.query(&create_table_query, &[]).await?;

        let max_blocknum: i32 = rows[0].get(0);
        println!("max_blocknum : {max_blocknum}");
        Ok(Some(max_blocknum))
    } else {
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} ( 
            id              SERIAL PRIMARY KEY,
            l2_output_root     VARCHAR NOT NULL,
            l2_block_hash   VARCHAR NOT NULL,
            l2_block_number  INTEGER NOT NULL,
            l1_transaction_hash    VARCHAR NOT NULL,
            l1_block_number    INTEGER NOT NULL,
            l1_transaction_index    INTEGER NOT NULL,
            l1_block_hash     VARCHAR NOT NULL
        )",
            table_name
        );
        client.execute(&create_table_query, &[]).await?;

        Ok(None)
    }
}

/// A function that inserts data into the postgres table
/// Parameters:
/// * table_name: The name of the postgres table
/// * client: The postgres client
/// * l2_output_root: The output root of the l2
/// * l2_block_hash: The block hash of the l2
/// * l2_block_number: The block number of the l2
/// * l1_timestamp: The timestamp of the l1
/// * l1_transaction_hash: The transaction hash of the l1
/// * l1_block_number: The block number of the l1
/// * l1_transaction_index: The transaction index of the l1
/// * l1_block_hash: The block hash of the l1
/// Returns:
/// Returns nothing except for error
pub async fn insert_into_postgres(
    table_name: String,
    client: &tokio_postgres::Client,
    params: ArbitrumParameters,
) -> Result<()> {
    let insert_query = format!("INSERT INTO {} (l2_output_root, l2_block_hash, l2_block_number, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash) VALUES ($1, $2, $3, $4, $5, $6, $7)", table_name);
    client
        .execute(
            &insert_query,
            &[
                &params.l2_output_root.to_string(),
                &params.l2_block_hash.to_string(),
                &(params.l2_block_number.as_u64() as i32),
                &params.l1_transaction_hash.to_string(),
                &(params.l1_block_number.as_u64() as i32),
                &(params.l1_transaction_index.as_u64() as i32),
                &params.l1_block_hash.to_string(),
            ],
        )
        .await?;

    Ok(())
}

pub async fn handle_arbitrum_events(
    log: &Log,
    chain_type: &ChainType,
) -> Result<ArbitrumParameters> {
    //? Example log : log = Log { address: 0x0b9857ae2d4a3dbe74ffe1d7df045bb7f96e4840, topics: [0xb4df3847300f076a369cd76d2314b470a1194d9e8a6bb97f1860aee88a5f6748, 0x46ac12a9031cfe15b510a19b1ee6a237409cb5659fba8a71192229f7d086e67f, 0xf4369a47ee900d312913d8cb382a4eb174272c42cead9cdaf8c4db9b5f0eb9e9], data: Bytes(0x), block_hash: Some(0x0bf39cb7a1ef70be6350438c8e99a22e785d46309c91aaaf65d760e92ed97bd7), block_number: Some(15843456), transaction_hash: Some(0x306ce7c969f40a8afc7dc2fa0a45ba13daee06fecbb1ed938c129749225a0963), transaction_index: Some(188), log_index: Some(323), transaction_log_index: None, log_type: None, removed: Some(false) }
    let l2_output_root = Bytes::from(log.topics[1].as_bytes().to_vec());
    let l2_block_hash = Bytes::from(log.topics[2].as_bytes().to_vec());
    let l1_transaction_hash = Bytes::from(log.transaction_hash.unwrap().as_bytes().to_vec());
    let l1_block_number = log.block_number.unwrap();
    let l1_transaction_index = log.transaction_index.unwrap();
    let l1_block_hash = Bytes::from(log.block_hash.unwrap().as_bytes().to_vec());
    let arbitrum_rpc_url = match chain_type {
        ChainType::Mainnet => std::env::var("ARBITRUM_MAINNET_RPC_URL")
            .expect("ARBITRUM_MAINNET_RPC_URL must be set."),
        ChainType::Sepolia => std::env::var("ARBITRUM_SEPOLIA_RPC_URL")
            .expect("ARBITRUM_SEPOLIA_RPC_URL must be set."),
        ChainType::Goerli => {
            std::env::var("ARBITRUM_GOERLI_RPC_URL").expect("ARBITRUM_GOERLI_RPC_URL must be set.")
        }
    };

    let arbitrum_fetcher = Fetcher::new(arbitrum_rpc_url.to_string());

    let block = arbitrum_fetcher
        .fetch_block_by_number(&l2_block_hash.to_string())
        .await?;

    let dec_number = u64::from_str_radix(&block.number.as_str()[2..], 16).unwrap();

    let l2_block_number: U256 = U256::from(dec_number);
    println!(
                "output_root = {l2_output_root}, l2blockhash = {l2_block_hash}, l2_block_number = {l2_block_number}, l1Blocknumber = {l1_block_number}, l1_transaction_hash={l1_transaction_hash}, l1_transaction_index={l1_transaction_index}, L1_block_hash={l1_block_hash}",
            );

    Ok(ArbitrumParameters {
        l2_output_root,
        l2_block_hash,
        l2_block_number,
        l1_transaction_hash,
        l1_block_number,
        l1_transaction_index,
        l1_block_hash,
    })
}
