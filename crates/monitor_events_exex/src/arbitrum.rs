use ::common::ChainType;
use eyre::Result;
use reth_primitives::{Bytes, U256, U64};

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
                l2_block_hash           VARCHAR NOT NULL,
                l2_block_number         INTEGER NOT NULL,
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
