use std::sync::Arc;

use ethers::{prelude::*, utils::*};

pub struct OPStackParameters {
    l2_output_root: Bytes,
    l1_game_address: String,
    l2_block_number: U256,
    l1_timestamp: U256,
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
pub async fn create_opstack_table_if_not_exists(
    table_name: String,
    client: &tokio_postgres::Client,
) -> Result<Option<i32>, tokio_postgres::Error> {
    let create_table_query = format!("SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '{}') AS table_existence;", table_name);
    let rows = client.query(&create_table_query, &[]).await?;

    // And then check that we got back the same string we sent over.
    let exist: bool = rows[0].get(0);
    println!("Table exist : {exist}");
    if exist {
        // Query the maximum l1_block_number
        let create_table_query = format!(
            "SELECT MAX(l1_block_number) as MaxBlock from {}",
            table_name
        );
        let rows = client.query(&create_table_query, &[]).await?;

        // Handle possible NULL result for max l1_block_number
        let max_blocknum: Option<i32> = rows[0].try_get(0)?;

        if let Some(max_num) = max_blocknum {
            println!("max_blocknum: {max_num}");
            Ok(Some(max_num))
        } else {
            println!("No entries in the table, hence no maximum block number.");
            Ok(None)
        }
    } else {
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} ( 
                id                      SERIAL PRIMARY KEY,
                l2_output_root          VARCHAR NOT NULL,
                l1_game_address         VARCHAR NOT NULL,
                l2_block_number         INTEGER NOT NULL,
                l1_timestamp            INTEGER NOT NULL,
                l1_transaction_hash     VARCHAR NOT NULL,
                l1_block_number         INTEGER NOT NULL,
                l1_transaction_index    INTEGER NOT NULL,
                l1_block_hash           VARCHAR NOT NULL
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
/// * l1_game_address: The contract address of FaultDisputeGame
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
    params: OPStackParameters,
) -> Result<(), tokio_postgres::Error> {
    let insert_query = format!("INSERT INTO {} (l2_output_root, l1_game_address, l2_block_number, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)", table_name);
    client
        .execute(
            &insert_query,
            &[
                &params.l2_output_root.to_string(),
                &(params.l1_game_address.to_string()),
                &(params.l2_block_number.as_u64() as i32),
                &(params.l1_timestamp.as_u64() as i32),
                &params.l1_transaction_hash.to_string(),
                &(params.l1_block_number.as_u64() as i32),
                &(params.l1_transaction_index.as_u64() as i32),
                &params.l1_block_hash.to_string(),
            ],
        )
        .await?;

    Ok(())
}

pub async fn handle_opstack_events(log: &Log, client: &Arc<Provider<Http>>) -> OPStackParameters {
    let l1_game_address = Address::from(log.topics[1]);
    let l2_output_root = Bytes::from(log.topics[3].as_bytes().to_vec());
    let l1_timestamp = U256::from_big_endian(&log.data[..]);
    let l1_transaction_hash = Bytes::from(log.transaction_hash.unwrap().as_bytes().to_vec());
    let l1_block_number = log.block_number.unwrap();
    let l1_transaction_index = log.transaction_index.unwrap();
    let l1_block_hash = Bytes::from(log.block_hash.unwrap().as_bytes().to_vec());

    let selector = id("l2BlockNumber()");

    let block_number = client
        .call_raw(
            &TransactionRequest::default()
                .from(H160::zero())
                .to(l1_game_address)
                .data(selector)
                .into(),
        )
        .await
        .unwrap();
    let l2_block_number = U256::from_big_endian(&block_number);

    //? We need to format the Address because without it, it is being inserted to db formatted like 0x1234…5678
    let formatted_address = format!("0x{}", hex::encode(l1_game_address.as_bytes()));

    println!(
        "output_root = {l2_output_root}, l1GameAddress = {formatted_address}, l2BlockNumber = {l2_block_number}, l1Blocknumber = {l1_block_number}, l1Timestamp = {l1_timestamp}, l1_transaction_hash={l1_transaction_hash}, l1_transaction_index={l1_transaction_index}, L1_block_hash={l1_block_hash}"
    );

    OPStackParameters {
        l2_output_root,
        l1_game_address: formatted_address,
        l2_block_number,
        l1_timestamp,
        l1_transaction_hash,
        l1_block_number,
        l1_transaction_index,
        l1_block_hash,
    }
}
