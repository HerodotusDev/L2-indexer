#[macro_use]
extern crate rocket;

use std::str::FromStr;

use common::Network;
use dotenv::dotenv;
use eyre::Result;
use rocket::form::{self, FromForm};
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;
use tokio_postgres::NoTls;

// Input for request parameters
#[derive(FromForm, Debug)]
pub struct ParamsInput {
    network: String,
    l2_block: i32,
}

#[derive(Serialize, Debug)]
enum OutputType {
    OpStack(OPStackParamsOutput),
    Arbitrum(ArbitrumParamsOutput),
}

// Output for request parameters of opstack
#[derive(Serialize, Debug)]
pub struct OPStackParamsOutput {
    l2_output_root: String,
    l2_output_index: i32,
    l2_block_number: i32,
    l1_timestamp: i32,
    l1_transaction_hash: String,
    l1_block_number: i32,
    l1_transaction_index: i32,
    l1_block_hash: String,
}

// Output for request parameters of arbitrum
#[derive(Serialize, Debug)]
pub struct ArbitrumParamsOutput {
    l2_output_root: String,
    l2_block_hash: String,
    l2_block_number: i32,
    l1_transaction_hash: String,
    l1_block_number: i32,
    l1_transaction_index: i32,
    l1_block_hash: String,
}

/// A function that connects to the postgres database
async fn connect_db(db_url: &str) -> Result<tokio_postgres::Client> {
    // Establish a PostgreSQL connection
    let (pg_client, connection) = tokio_postgres::connect(db_url, NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    Ok(pg_client)
}

#[derive(Serialize, Debug)]
struct HighestBlock {
    chain: String,
    block_number: i32,
}

async fn handle_get_highest_l2_block(
    network: &str,
    pg_client: &tokio_postgres::Client,
) -> Result<HighestBlock> {
    let network = Network::from_str(network)
        .map_err(|e| eyre::eyre!("Invalid Network: {:?}", e.to_string()))?;

    let max_query = format!(
        "SELECT max(blocks.l2_block_number) FROM public.{} blocks",
        network.to_string()
    );

    let rows = pg_client.query(&max_query, &[]).await?;
    if rows.is_empty() {
        Err(eyre::eyre!("Expected at least 1 row"))
    } else {
        let block_number: i32 = rows[0].get(0);

        Ok(HighestBlock {
            chain: network.to_string(),
            block_number,
        })
    }
}

/// A function that gets the output root from a block number query from postgres db
async fn handle_query_opstack(
    params: &ParamsInput,
    pg_client: &tokio_postgres::Client,
) -> Result<(String, i32, i32, i32, String, i32, i32, String)> {
    let ParamsInput { l2_block, network } = params;
    let network = Network::from_str(network)
        .map_err(|e| eyre::eyre!("Invalid Network: {:?}", e.to_string()))?;

    let select_query = format!("SELECT l2_output_root, l2_output_index, l2_block_number, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash
    FROM {} 
    WHERE l2_block_number >= $1
    ORDER BY l2_block_number ASC
    LIMIT 1;", network.to_string());

    let rows = pg_client.query(&select_query, &[&l2_block]).await?;
    if rows.is_empty() {
        Err(eyre::eyre!("Expected at least 1 row"))
    } else {
        // Get both output_root and l2_blocknum from the query result
        let l2_output_root: String = rows[0].get(0);
        let l2_output_index: i32 = rows[0].get(1);
        let l2_block_number: i32 = rows[0].get(2);
        let l1_timestamp: i32 = rows[0].get(3);
        let l1_transaction_hash: String = rows[0].get(4);
        let l1_block_number: i32 = rows[0].get(5);
        let l1_transaction_index: i32 = rows[0].get(6);
        let l1_block_hash: String = rows[0].get(7);

        println!("L2 output root: {}", l2_output_root);
        println!("L2 output index: {}", l2_output_index);
        println!("L2 block number: {}", l2_block_number);
        println!("L1 timestamp: {}", l1_timestamp);
        println!("L1 transaction hash: {}", l1_transaction_hash);
        println!("L1 block number: {}", l1_block_number);
        println!("L1 transaction index: {}", l1_transaction_index);
        println!("L1 block hash: {}", l1_block_hash);

        Ok((
            l2_output_root,
            l2_output_index,
            l2_block_number,
            l1_timestamp,
            l1_transaction_hash,
            l1_block_number,
            l1_transaction_index,
            l1_block_hash,
        ))
    }
}

/// A function that gets the output root from a block number query from postgres db
async fn handle_query_arbitrum(
    params: &ParamsInput,
    pg_client: &tokio_postgres::Client,
) -> Result<(String, String, i32, String, i32, i32, String)> {
    let l2_block = params.l2_block;
    let network = &params.network;
    let select_query = format!("SELECT l2_output_root, l2_block_hash, l2_block_number, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash
    FROM {} 
    WHERE l2_block_number >= $1
    ORDER BY l2_block_number ASC
    LIMIT 1;", network);

    let rows = pg_client.query(&select_query, &[&l2_block]).await?;
    if rows.is_empty() {
        Err(eyre::eyre!("Expected at least 1 row"))
    } else {
        // Get both output_root and l2_blocknum from the query result
        let l2_output_root: String = rows[0].get(0);
        let l2_block_hash: String = rows[0].get(1);
        let l2_block_number: i32 = rows[0].get(2);
        let l1_transaction_hash: String = rows[0].get(3);
        let l1_block_number: i32 = rows[0].get(4);
        let l1_transaction_index: i32 = rows[0].get(5);
        let l1_block_hash: String = rows[0].get(6);

        println!("L2 output root: {}", l2_output_root);
        println!("L2 block hash: {}", l2_block_hash);
        println!("L2 block number: {}", l2_block_number);
        println!("L1 transaction hash: {}", l1_transaction_hash);
        println!("L1 block number: {}", l1_block_number);
        println!("L1 transaction index: {}", l1_transaction_index);
        println!("L1 block hash: {}", l1_block_hash);

        Ok((
            l2_output_root,
            l2_block_hash,
            l2_block_number,
            l1_transaction_hash,
            l1_block_number,
            l1_transaction_index,
            l1_block_hash,
        ))
    }
}

// Input for request parameters
#[derive(FromForm, Debug)]
pub struct GetHighestL2BlockParamsInput {
    network: String,
}

#[get("/highest-l2-block?<query..>")]
async fn get_highest_l2_block(
    query: form::Result<'_, GetHighestL2BlockParamsInput>,
) -> Result<Json<HighestBlock>, status::Conflict<std::string::String>> {
    let params = query.map_err(|e| status::Conflict(e.to_string()))?;
    dotenv().ok();
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set");
    let pg_client = connect_db(db_url).await.unwrap();
    let network: &str = &params.network;

    match handle_get_highest_l2_block(network, &pg_client).await {
        Ok(highest_blocks) => Ok(Json(highest_blocks)),
        Err(e) => Err(status::Conflict(e.to_string())),
    }
}

#[get("/output-root?<query..>")]
async fn get_output_root(
    query: form::Result<'_, ParamsInput>,
) -> Result<Json<OutputType>, status::Conflict<std::string::String>> {
    let params = query.map_err(|e| status::Conflict(e.to_string()))?;
    dotenv().ok();
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set");
    let pg_client = connect_db(db_url).await.unwrap();
    let network: &str = &params.network;
    match network {
        "arbitrum_mainnet" | "arbitrum_sepolia" => {
            match handle_query_arbitrum(&params, &pg_client).await {
                Ok((
                    l2_output_root,
                    l2_block_hash,
                    l2_block_number,
                    l1_transaction_hash,
                    l1_block_number,
                    l1_transaction_index,
                    l1_block_hash,
                )) => Ok(Json(OutputType::Arbitrum(ArbitrumParamsOutput {
                    l2_output_root,
                    l2_block_hash,
                    l2_block_number,
                    l1_transaction_hash,
                    l1_block_number,
                    l1_transaction_index,
                    l1_block_hash,
                }))),
                Err(e) => Err(status::Conflict(e.to_string())),
            }
        }
        _ => match handle_query_opstack(&params, &pg_client).await {
            Ok((
                l2_output_root,
                l2_output_index,
                l2_block_number,
                l1_timestamp,
                l1_transaction_hash,
                l1_block_number,
                l1_transaction_index,
                l1_block_hash,
            )) => Ok(Json(OutputType::OpStack(OPStackParamsOutput {
                l2_output_root,
                l2_output_index,
                l2_block_number,
                l1_transaction_hash,
                l1_transaction_index,
                l1_timestamp,
                l1_block_number,
                l1_block_hash,
            }))),
            Err(e) => Err(status::Conflict(e.to_string())),
        },
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_output_root, get_highest_l2_block])
}
