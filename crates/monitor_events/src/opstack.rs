use common::{get_network_config, ChainType, Network};

use crate::fetcher::Fetcher;
use ethers::prelude::*;

abigen!(DisputeGame, "abi/DisputeGame.json");
use eyre::{eyre, Result};
use std::str::FromStr;
use std::{convert::TryInto, sync::Arc};

fn parse_bytes(name: &str, s: &str) -> eyre::Result<Bytes> {
    Bytes::from_str(s).map_err(|_| eyre!("invalid {name} hex: {s}"))
}

pub struct OPStackParameters {
    l2_output_root: Bytes,
    l2_output_index: U256,
    l2_block_number: U256,
    l1_timestamp: U256,
    l1_transaction_hash: Bytes,
    l1_block_number: U64,
    l1_transaction_index: U64,
    l1_block_hash: Bytes,
}

pub struct OPStackDisputeGameParameters {
    game_index: u64,
    game_address: Address,
    game_type: u32,
    timestamp: u64,
    root_claim: Bytes,
    game_state: u64,
    proposer_address: Address,
    l2_block_number: U256,             // Keep original for reference
    l2_block_number_safe: Option<u64>, // Safe u64 version for database
    l2_state_root: Option<Bytes>,
    l2_withdrawal_storage_root: Option<Bytes>,
    l2_block_hash: Option<Bytes>,
    l1_timestamp: U64,
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
                l2_output_index         INTEGER NOT NULL,
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

/// A function that creates a table if it doesn't exist, and returns the max block number in the table if it exists.
/// Parameters:
/// * table_name: The name of the postgres table
/// * client: The postgres client
/// Returns:
/// * Option<i32>: The max block number in the table if it exists, otherwise None
pub async fn create_opstack_dispute_games_table_if_not_exists(
    table_name: String,
    client: &tokio_postgres::Client,
) -> Result<Option<i64>, tokio_postgres::Error> {
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
        let max_blocknum: Option<i64> = rows[0].try_get(0)?;

        if let Some(max_num) = max_blocknum {
            println!("max_blocknum: {max_num}");
            Ok(Some(max_num))
        } else {
            println!("No entries in the table, hence no maximum block number.");
            Ok(None)
        }
    } else {
        // l2_state_root, l2_withdrawal_storage_root, l2_block_hash can be null if there is worng game created for nonexistent L2 block
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id                      SERIAL PRIMARY KEY,
                game_index          BIGINT NOT NULL,
                game_address          VARCHAR NOT NULL,
                game_type         BIGINT NOT NULL,
                timestamp         BIGINT NOT NULL,
                root_claim            VARCHAR NOT NULL,
                game_state         BIGINT NOT NULL,
                proposer_address            VARCHAR NOT NULL,
                l2_block_number     BIGINT NOT NULL,
                l2_state_root     VARCHAR,
                l2_withdrawal_storage_root     VARCHAR,
                l2_block_hash     VARCHAR,
                l1_timestamp            BIGINT NOT NULL,
                l1_transaction_hash     VARCHAR NOT NULL,
                l1_block_number         BIGINT NOT NULL,
                l1_transaction_index    BIGINT NOT NULL,
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
/// * l2_output_index: The output index of the l2
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
    let insert_query = format!("INSERT INTO {} (l2_output_root, l2_output_index, l2_block_number, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)", table_name);
    client
        .execute(
            &insert_query,
            &[
                &params.l2_output_root.to_string(),
                &(params.l2_output_index.as_u64() as i32),
                &(params.l2_block_number.try_into().unwrap_or(0u64) as i32),
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

pub async fn insert_fdg_into_postgres(
    table_name: String,
    client: &tokio_postgres::Client,
    params: OPStackDisputeGameParameters,
) -> Result<(), tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} (
            game_index,
            game_address,
            game_type,
            timestamp,
            root_claim,
            game_state,
            proposer_address,
            l2_block_number,
            l2_state_root,
            l2_withdrawal_storage_root,
            l2_block_hash,
            l1_timestamp,
            l1_transaction_hash,
            l1_block_number,
            l1_transaction_index,
            l1_block_hash
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16
        )",
        table_name
    );

    let game_index_i64 = params.game_index as i64;
    //let game_address_str = params.game_address.to_string();
    let game_address_str = format!("{:#x}", params.game_address);
    let game_type_i32 = params.game_type as i64;
    let timestamp_i64 = params.timestamp as i64;
    let root_claim_str = format!("{:#x}", params.root_claim);
    let game_state_i64 = params.game_state as i64;

    //let proposer_address_str = params.proposer_address.to_string();
    let proposer_address_str = format!("{:#x}", params.proposer_address);

    let l2_block_number_i64 = params.l2_block_number_safe.unwrap_or(0) as i64;

    // Log if we're using a fallback value
    if params.l2_block_number_safe.is_none() {
        eprintln!(
            "Using fallback L2 block number 0 for dispute game {} (original: {})",
            params.game_index, params.l2_block_number
        );
    }

    let l2_state_root_hex_str: Option<String> =
        params.l2_state_root.as_ref().map(|h| format!("{:#x}", h));
    let l2_withdrawal_storage_root_hex_str: Option<String> = params
        .l2_withdrawal_storage_root
        .as_ref()
        .map(|h| format!("{:#x}", h));
    let l2_block_hash_hex_str: Option<String> =
        params.l2_block_hash.as_ref().map(|h| format!("{:#x}", h));

    // let l2_state_root_str = format!("{:#x}", params.l2_state_root);
    // let l2_withdrawal_storage_root_str = format!("{:#x}", params.l2_withdrawal_storage_root);
    // let l2_block_hash_str = format!("{:#x}", params.l2_block_hash);

    let l1_timestamp_i64 = params.l1_timestamp.as_u64() as i64;
    let l1_tx_hash_str = params.l1_transaction_hash.to_string();
    let l1_block_number_i64 = params.l1_block_number.as_u64() as i64;
    let l1_tx_index_i64 = params.l1_transaction_index.as_u64() as i64;
    let l1_block_hash_str = params.l1_block_hash.to_string();

    client
        .execute(
            &insert_query,
            &[
                &game_index_i64,
                &game_address_str,
                &game_type_i32,
                &timestamp_i64,
                &root_claim_str,
                &game_state_i64,
                &proposer_address_str,
                &l2_block_number_i64,
                &l2_state_root_hex_str,
                &l2_withdrawal_storage_root_hex_str,
                &l2_block_hash_hex_str,
                &l1_timestamp_i64,
                &l1_tx_hash_str,
                &l1_block_number_i64,
                &l1_tx_index_i64,
                &l1_block_hash_str,
            ],
        )
        .await?;

    Ok(())
}

pub async fn get_highest_game_index(
    table_name: &str,
    client: &tokio_postgres::Client,
) -> Result<u64, tokio_postgres::Error> {
    let query = format!("SELECT MAX(game_index) FROM {}", table_name);
    let rows = client.query(&query, &[]).await?;

    if let Some(row) = rows.get(0) {
        let max_index: Option<i64> = row.get(0);
        Ok(max_index.unwrap_or(0).max(0) as u64)
    } else {
        Ok(0)
    }
}

pub fn handle_opstack_events(log: &Log) -> OPStackParameters {
    let l2_output_root = Bytes::from(log.topics[1].as_bytes().to_vec());
    let l2_output_index = U256::from_big_endian(log.topics[2].as_bytes());
    let l2_block_number = U256::from_big_endian(log.topics[3].as_bytes());
    let l1_timestamp = U256::from_big_endian(&log.data[..]);
    let l1_transaction_hash = Bytes::from(log.transaction_hash.unwrap().as_bytes().to_vec());
    let l1_block_number = log.block_number.unwrap();
    let l1_transaction_index = log.transaction_index.unwrap();
    let l1_block_hash = Bytes::from(log.block_hash.unwrap().as_bytes().to_vec());

    println!(
        "output_root = {l2_output_root}, l2OutputIndex = {l2_output_index}, l2BlockNumber = {l2_block_number}, l1Blocknumber = {l1_block_number}, l1Timestamp = {l1_timestamp}, l1_transaction_hash={l1_transaction_hash}, l1_transaction_index={l1_transaction_index}, L1_block_hash={l1_block_hash}"
    );

    OPStackParameters {
        l2_output_root,
        l2_output_index,
        l2_block_number,
        l1_timestamp,
        l1_transaction_hash,
        l1_block_number,
        l1_transaction_index,
        l1_block_hash,
    }
}

pub async fn handle_opstack_fdg_events(
    log: &Log,
    network: &Network,
    l1_provider: Arc<Provider<Http>>,
    game_index: u64,
) -> Result<OPStackDisputeGameParameters, eyre::Error> {
    let dispute_proxy_address: Address = Address::from(log.topics[1]).into();
    let game_type = {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(log.topics[2].as_bytes());
        U256::from_big_endian(&bytes).as_u32()
    };
    let root_claim = Bytes::from(log.topics[3].as_bytes().to_vec());

    let dispute_game = DisputeGame::new(dispute_proxy_address, l1_provider.clone());

    let status_u8: u8 = dispute_game.status().call().await.map_err(|e| {
        eprintln!("status() failed: {e:?}");
        eyre!("status() failed: {e:?}")
    })?;
    let game_status: u64 = status_u8 as u64;

    let timestamp: u64 = dispute_game.created_at().call().await.map_err(|e| {
        eprintln!("created_at() failed: {e:?}");
        eyre!("created_at() failed: {e:?}")
    })?;

    // This is bacause OP Sepolia game contract dont have the gemaCreator method
    let mut game_creator: Address =
        Address::from_str("0x0000000000000000000000000000000000000000").unwrap();

    if network.chain_type == ChainType::Mainnet {
        game_creator = dispute_game.game_creator().call().await.map_err(|e| {
            eprintln!("game_creator() failed: {e:?}");
            eyre!("game_creator() failed: {e:?}")
        })?;
    }

    let network_config = get_network_config(network.chain_type, network.chain_name);

    let trusted_proposer: Option<Address> = network_config
        .trusted_proposer_address
        .as_deref()
        .and_then(|s| s.parse::<Address>().ok());

    let is_trusted_proposer = match trusted_proposer {
        Some(addr) => game_creator == addr,
        None => false,
    };

    println!(
        "Fetched dispute game at address {:#x}",
        dispute_proxy_address
    );

    if !(game_status == 2 || (is_trusted_proposer && (game_status == 0 || game_status == 2))) {
        // If the dispute game is not finalised, we anyway inserting it to the db with correct state
        // Later, in db retrieval state wi checking this condition also
        println!(
           "Dispute game not finalized (game_status != 2 and not trusted proposer with game_status 0 or 2)"
        );
        //return Err(eyre::eyre!("Dispute game not finalized (status != 2 and not trusted proposer with status 0 or 2)"));
    }

    let l2_block_number: U256 = dispute_game.l_2_block_number().call().await.map_err(|e| {
        eprintln!("l2_block_number() failed: {e:?}");
        eyre!("l2_block_number() failed: {e:?}")
    })?;

    // Check if L2 block number is within u64 range before proceeding
    let l2_block_number_u64: u64 = match l2_block_number.try_into() {
        Ok(num) => num,
        Err(_) => {
            eprintln!(
                "L2 block number {} is too large for u64 (max u64: {}), skipping L2 data fetch",
                l2_block_number,
                u64::MAX
            );
            // Return early with None values for L2 data
            return Ok(OPStackDisputeGameParameters {
                game_index,
                game_address: dispute_proxy_address,
                game_type,
                timestamp,
                root_claim,
                game_state: game_status,
                proposer_address: game_creator,
                l2_block_number,
                l2_block_number_safe: None, // No safe version available
                l2_state_root: None,
                l2_withdrawal_storage_root: None,
                l2_block_hash: None,
                l1_timestamp: U64::from_big_endian(&log.data[..]),
                l1_transaction_hash: Bytes::from(log.transaction_hash.unwrap().as_bytes().to_vec()),
                l1_block_number: log.block_number.unwrap(),
                l1_transaction_index: log.transaction_index.unwrap(),
                l1_block_hash: Bytes::from(log.block_hash.unwrap().as_bytes().to_vec()),
            });
        }
    };

    // Get the L2 block details from L2 RPC
    let l2_rpc_url = std::env::var("L2_RPC_URL").expect("L2_RPC_URL must be set.");

    let l2_rpc_fetcher = Fetcher::new(l2_rpc_url.to_string());

    let l2_block_number_hex = format!("0x{:x}", l2_block_number_u64);
    //let optimism_output = l2_rpc_fetcher.fetch_optimism_output_at_block(&l2_block_number_hex).await.unwrap();
    // let optimism_output = match l2_rpc_fetcher.fetch_optimism_output_at_block(&l2_block_number_hex).await? {
    //     Some(out) => { /* process */ }
    //     None => {
    //         // block/output not available → skip insert or insert NULL/defaults
    //     }
    // };

    // let l2_state_root: Bytes =
    //     Bytes::from_str(&optimism_output.state_root).expect("Invalid state_root hex");

    // let l2_withdrawal_storage_root: Bytes =
    //     Bytes::from_str(&optimism_output.withdrawal_storage_root)
    //         .expect("Invalid withdrawal_storage_root hex");

    // let l2_block_hash: Bytes =
    //     Bytes::from_str(&optimism_output.block_ref.hash).expect("Invalid block hash hex");
    //
    let maybe_out = l2_rpc_fetcher
        .fetch_optimism_output_at_block(&l2_block_number_hex)
        .await?;

    let (l2_state_root, l2_withdrawal_storage_root, l2_block_hash) = match maybe_out {
        Some(out) => (
            Some(
                parse_bytes("state_root", &out.state_root)
                    .map_err(|e| eyre!("Failed to parse state_root: {}", e))?,
            ),
            Some(
                parse_bytes("withdrawal_storage_root", &out.withdrawal_storage_root)
                    .map_err(|e| eyre!("Failed to parse withdrawal_storage_root: {}", e))?,
            ),
            Some(
                parse_bytes("block_hash", &out.block_ref.hash)
                    .map_err(|e| eyre!("Failed to parse block_hash: {}", e))?,
            ),
        ),
        None => (None, None, None),
    };

    // Example insert (VARCHAR columns; store hex strings). Assumes columns are NULLable.
    // let l2_state_root_hex: Option<String> =
    //     l2_state_root.as_ref().map(|b| format!("0x{}", hex::encode(b)));
    // let l2_withdrawal_storage_root_hex: Option<String> =
    //     l2_withdrawal_storage_root.as_ref().map(|b| format!("0x{}", hex::encode(b)));
    // let l2_block_hash_hex: Option<String> =
    //     l2_block_hash.as_ref().map(|b| format!("0x{}", hex::encode(b)));

    let l1_timestamp = U64::from_big_endian(&log.data[..]);
    let l1_transaction_hash = Bytes::from(log.transaction_hash.unwrap().as_bytes().to_vec());
    let l1_block_number = log.block_number.unwrap();
    let l1_transaction_index = log.transaction_index.unwrap();
    let l1_block_hash = Bytes::from(log.block_hash.unwrap().as_bytes().to_vec());

    Ok(OPStackDisputeGameParameters {
        game_index,
        game_address: dispute_proxy_address,
        game_type,
        timestamp,
        root_claim,
        game_state: game_status,
        proposer_address: game_creator,
        l2_block_number,
        l2_block_number_safe: Some(l2_block_number_u64), // Use the safe u64 version
        l2_state_root,
        l2_withdrawal_storage_root,
        l2_block_hash,
        l1_timestamp,
        l1_transaction_hash,
        l1_block_number,
        l1_transaction_index,
        l1_block_hash,
    })
}
