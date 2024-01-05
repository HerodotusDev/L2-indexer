use arbitrum::create_arbitrum_table_if_not_exists;
use config::{Config, File, FileFormat};
use dotenv::dotenv;
use ethers::prelude::*;
use eyre::Result;
use opstack::create_opstack_table_if_not_exists;
use serde::Deserialize;
use std::{
    str::FromStr,
    sync::Arc,
    thread,
    time::{self, Duration},
};
use tokio_postgres::NoTls;

use crate::{
    arbitrum::handle_arbitrum_events,
    opstack::{handle_opstack_events, insert_into_postgres},
};

mod arbitrum;
mod opstack;

/// A struct that represents the Networks struct in the JSON file
#[derive(Debug, Deserialize)]
struct Networks {
    name: String,
    l1_contract: String,
    block_delay: u64,
    poll_period_sec: u64,
    batch_size: Option<u64>,
}

/// A chain type
enum ChainType {
    Arbitrum,
    Opstack,
}

impl FromStr for ChainType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arbitrum" => Ok(ChainType::Arbitrum),
            "opstack" => Ok(ChainType::Opstack),
            _ => Err(anyhow::anyhow!("invalid chain ")),
        }
    }
}

/// A builder that gets config from JSON and returns Config.
/// Parameters:
/// * network_config: The name of the network want to get from JSON
/// Returns:
/// * Config struct that contains all the config data
fn make(network_config: &str) -> Config {
    let config_name = format!("crates/monitor_events/networks/{}", network_config);
    Config::builder()
        .add_source(File::new(&config_name, FileFormat::Json))
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Settup the environment variables
    dotenv().ok();
    let rpc_url: &str = &std::env::var("RPC_URL").expect("RPC_URL must be set.");
    let parse_type: ChainType =
        ChainType::from_str(&std::env::var("TYPE").expect("TYPE must be set.")).unwrap();
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set.");
    let network_config: &str = &std::env::var("NETWORK").expect("NETWORK must be set.");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);
    let config = make(network_config);
    let network: Networks = config.try_deserialize().unwrap();
    let _block_delay = network.block_delay;
    let _poll_period_sec = network.poll_period_sec;
    let table_name = network.name;
    let block_delay: U64 = U64([_block_delay]);
    let poll_period_sec: Duration = time::Duration::from_secs(_poll_period_sec);
    let address: Address = network.l1_contract.parse()?;

    // Set block number values to filter
    let mut new_block_num = client.get_block_number().await? - block_delay;
    let batch_size = network.batch_size.unwrap_or(new_block_num.as_u64());

    // Establish a PostgreSQL connection
    let (pg_client, connection) = tokio_postgres::connect(db_url, NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    let mut from_block_num = match parse_type {
        ChainType::Opstack =>
        // Create a table if it doesn't exist
        {
            match create_opstack_table_if_not_exists(table_name.clone(), &pg_client).await {
                Ok(table_result) => match table_result {
                    Some(max_blocknumber) => (max_blocknumber + 1).into(),
                    None => U64([0]),
                },
                Err(err) => panic!("Error creating table: {:?}", err),
            }
        }
        ChainType::Arbitrum => {
            match create_arbitrum_table_if_not_exists(table_name.clone(), &pg_client).await {
                Ok(table_result) => match table_result {
                    Some(max_blocknumber) => (max_blocknumber + 1).into(),
                    None => U64([0]),
                },
                Err(err) => panic!("Error creating table: {:?}", err),
            }
        }
    };

    let mut filter = match parse_type {
        ChainType::Opstack => Filter::new()
            .address(address)
            .event("OutputProposed(bytes32,uint256,uint256,uint256)")
            .from_block(from_block_num)
            .to_block(new_block_num),
        ChainType::Arbitrum => Filter::new()
            .address(address)
            .event("SendRootUpdated(bytes32,bytes32)")
            .from_block(from_block_num)
            .to_block(new_block_num),
    };

    // Loop to get the logs with time gap and with batch
    loop {
        let block_gap = new_block_num.as_u64() - from_block_num.as_u64();
        let upper_limit = if block_gap > batch_size {
            from_block_num.as_u64() + batch_size as u64 - 1
        } else {
            new_block_num.as_u64()
        };

        filter = filter
            .from_block(from_block_num)
            .to_block(U64([upper_limit]));

        let logs = client.get_logs(&filter).await?;
        println!(
            "from {from_block_num:?} to {new_block_num:?}, {} pools found!",
            logs.iter().len()
        );

        for log in logs.iter() {
            match parse_type {
                ChainType::Opstack => {
                    let params = handle_opstack_events(log);
                    // Insert the data into PostgreSQL
                    if let Err(err) =
                        insert_into_postgres(table_name.clone(), &pg_client, params).await
                    {
                        eprintln!("Error inserting data into PostgreSQL: {:?}", err);
                    }
                }
                ChainType::Arbitrum => {
                    let params = handle_arbitrum_events(log);
                    // Insert the data into PostgreSQL
                    if let Err(err) =
                        arbitrum::insert_into_postgres(table_name.clone(), &pg_client, params).await
                    {
                        eprintln!("Error inserting data into PostgreSQL: {:?}", err);
                    }
                }
            }
        }
        // If we are in batch mode, don't sleep, and prepare for the next batch
        if block_gap > batch_size {
            from_block_num = U64([upper_limit + 1]);
        } else {
            thread::sleep(poll_period_sec);
            from_block_num = U64([upper_limit + 1]);
            new_block_num = client.get_block_number().await? - block_delay;
        }
    }
}
