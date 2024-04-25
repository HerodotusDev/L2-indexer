use ::common::{ChainName, ChainType};
use arbitrum::create_arbitrum_table_if_not_exists;
use common::Network;
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

use crate::{arbitrum::handle_arbitrum_events, opstack::handle_opstack_events};

mod arbitrum;
mod fetcher;
mod opstack;

/// A struct that represents the Networks struct in the JSON file
#[derive(Debug, Deserialize)]
struct Networks {
    name: String,
    l1_contract: String,
    l1_contract_deployment_block: u64,
    block_delay: u64,
    poll_period_sec: u64,
    batch_size: Option<u64>,
}

/// A builder that gets config from JSON and returns Config.
/// Parameters:
/// * network_config: The name of the network want to get from JSON
/// Returns:
/// * Networks struct that contains all the network config data
fn get_network_config(chain_type: ChainType, chain_name: ChainName) -> Networks {
    let network = Network {
        chain_name,
        chain_type,
    };
    let config_name = format!(
        "crates/monitor_events/networks/{}",
        network.to_string().to_lowercase()
    );
    let config = Config::builder()
        .add_source(File::new(&config_name, FileFormat::Json))
        .build()
        .unwrap();
    config.try_deserialize().unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Settup the environment variables
    dotenv().ok();
    let rpc_url: &str = &std::env::var("RPC_URL").expect("RPC_URL must be set.");
    let chain_type: ChainType =
        ChainType::from_str(&std::env::var("CHAIN_TYPE").expect("TYPE must be set.")).unwrap();
    let chain_name: ChainName =
        ChainName::from_str(&std::env::var("CHAIN_NAME").expect("TYPE must be set.")).unwrap();
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set.");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);
    let network = get_network_config(chain_type, chain_name);
    let block_delay = network.block_delay;
    let poll_period_sec = network.poll_period_sec;
    let table_name = network.name;
    let block_delay: U64 = U64([block_delay]);
    let poll_period_sec: Duration = time::Duration::from_secs(poll_period_sec);
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

    let mut from_block_num = match chain_name {
        ChainName::Optimism | ChainName::Base | ChainName::Zora => {
            create_opstack_table_if_not_exists(table_name.clone(), &pg_client).await
        }
        ChainName::Arbitrum => {
            create_arbitrum_table_if_not_exists(table_name.clone(), &pg_client).await
        }
    }
    .expect("Error creating table")
    .map_or(
        U64([network.l1_contract_deployment_block]),
        |max_blocknumber| (max_blocknumber + 1).into(),
    );

    let event_signature = match chain_name {
        ChainName::Optimism | ChainName::Base | ChainName::Zora => {
            "OutputProposed(bytes32,uint256,uint256,uint256)"
        }
        ChainName::Arbitrum => "SendRootUpdated(bytes32,bytes32)",
    };

    let mut filter = Filter::new()
        .event(event_signature)
        .address(address)
        .from_block(from_block_num)
        .to_block(new_block_num);

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
            match chain_name {
                ChainName::Optimism | ChainName::Base | ChainName::Zora => {
                    let params = handle_opstack_events(log);
                    // Insert the data into PostgreSQL
                    if let Err(err) =
                        opstack::insert_into_postgres(table_name.clone(), &pg_client, params).await
                    {
                        eprintln!("Error inserting data into PostgreSQL: {:?}", err);
                    }
                }
                ChainName::Arbitrum => {
                    let params = handle_arbitrum_events(log, &chain_type).await.unwrap();
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
