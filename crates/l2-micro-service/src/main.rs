#[macro_use]
extern crate rocket;

use eyre::Result;

use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;

use dotenv::dotenv;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

// Input for request parameters
#[derive(Deserialize, Debug)]
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
    l2_blocknumber: i32,
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

#[rocket::async_trait]
impl<'r> FromData<'r> for ParamsInput {
    type Error = std::io::Error;

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> rocket::data::Outcome<'r, Self> {
        let mut string = String::new();
        if let Err(e) = data.open(512.kilobytes()).read_to_string(&mut string).await {
            return Outcome::Failure((Status::InternalServerError, e));
        }

        let params: ParamsInput = match serde_json::from_str(&string) {
            Ok(params) => params,
            Err(e) => {
                return Outcome::Failure((
                    Status::UnprocessableEntity,
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                ))
            }
        };

        Outcome::Success(params)
    }
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

/// A function that gets the output root from a block number query from postgres db
async fn handle_query_opstack(
    params: &ParamsInput,
    pg_client: &tokio_postgres::Client,
) -> Result<(String, i32, i32, i32, String, i32, i32, String)> {
    let l2_block = params.l2_block;
    let network = &params.network;
    let select_query = format!("SELECT l2_output_root, l2_output_index, l2_blocknumber, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash
    FROM {} 
    WHERE l2_blocknumber >= $1
    ORDER BY l2_blocknumber ASC
    LIMIT 1;", network);

    let rows = pg_client.query(&select_query, &[&l2_block]).await?;
    if rows.is_empty() {
        Err(eyre::eyre!("Expected at least 1 row"))
    } else {
        // Get both output_root and l2_blocknum from the query result
        let l2_output_root: String = rows[0].get(0);
        let l2_output_index: i32 = rows[0].get(1);
        let l2_blocknumber: i32 = rows[0].get(2);
        let l1_timestamp: i32 = rows[0].get(3);
        let l1_transaction_hash: String = rows[0].get(4);
        let l1_block_number: i32 = rows[0].get(5);
        let l1_transaction_index: i32 = rows[0].get(6);
        let l1_block_hash: String = rows[0].get(7);

        println!("L2 output root: {}", l2_output_root);
        println!("L2 output index: {}", l2_output_index);
        println!("L2 block number: {}", l2_blocknumber);
        println!("L1 timestamp: {}", l1_timestamp);
        println!("L1 transaction hash: {}", l1_transaction_hash);
        println!("L1 block number: {}", l1_block_number);
        println!("L1 transaction index: {}", l1_transaction_index);
        println!("L1 block hash: {}", l1_block_hash);

        Ok((
            l2_output_root,
            l2_output_index,
            l2_blocknumber,
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

#[post("/output-root", format = "json", data = "<params>")]
async fn get_output_root(
    params: ParamsInput,
) -> Result<Json<OutputType>, status::Conflict<std::string::String>> {
    dotenv().ok();
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set");
    let pg_client = connect_db(db_url).await.unwrap();
    let network: &str = &params.network;
    match network {
        "arbitrum_mainnet" => match handle_query_arbitrum(&params, &pg_client).await {
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
            Err(e) => Err(status::Conflict(Some(e.to_string()))),
        },
        _ => match handle_query_opstack(&params, &pg_client).await {
            Ok((
                l2_output_root,
                l2_output_index,
                l2_blocknumber,
                l1_timestamp,
                l1_transaction_hash,
                l1_block_number,
                l1_transaction_index,
                l1_block_hash,
            )) => Ok(Json(OutputType::OpStack(OPStackParamsOutput {
                l2_output_root,
                l2_output_index,
                l2_blocknumber,
                l1_transaction_hash,
                l1_transaction_index,
                l1_timestamp,
                l1_block_number,
                l1_block_hash,
            }))),
            Err(e) => Err(status::Conflict(Some(e.to_string()))),
        },
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_output_root])
}
