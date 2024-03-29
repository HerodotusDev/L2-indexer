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
mod fetcher;
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

/// A chain name
#[derive(Debug, Clone, Copy)]
enum ChainName {
    Arbitrum,
    Base,
    Optimism,
    Zora,
}

impl ToString for ChainName {
    fn to_string(&self) -> String {
        match self {
            ChainName::Arbitrum => "arbitrum".to_string(),
            ChainName::Base => "base".to_string(),
            ChainName::Optimism => "optimism".to_string(),
            ChainName::Zora => "zora".to_string(),
        }
    }
}

impl FromStr for ChainName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arbitrum" => Ok(ChainName::Arbitrum),
            "base" => Ok(ChainName::Base),
            "optimism" => Ok(ChainName::Optimism),
            "zora" => Ok(ChainName::Zora),
            _ => Err(eyre::eyre!("invalid chain name")),
        }
    }
}

/// A chain name
#[derive(Debug, Clone, Copy)]
enum ChainType {
    Mainnet,
    Goerli,
    Sepolia,
}

impl ToString for ChainType {
    fn to_string(&self) -> String {
        match self {
            ChainType::Mainnet => "mainnet".to_string(),
            ChainType::Goerli => "goerli".to_string(),
            ChainType::Sepolia => "sepolia".to_string(),
        }
    }
}

impl FromStr for ChainType {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(ChainType::Mainnet),
            "goerli" => Ok(ChainType::Goerli),
            "sepolia" => Ok(ChainType::Sepolia),
            _ => Err(eyre::eyre!("invalid chain type")),
        }
    }
}

/// A builder that gets config from JSON and returns Config.
/// Parameters:
/// * network_config: The name of the network want to get from JSON
/// Returns:
/// * Networks struct that contains all the network config data
fn get_network_config(chain_type: ChainType, chain_name: ChainName) -> Networks {
    let config_name = format!(
        "crates/monitor_events/networks/{}_{}",
        chain_name.to_string(),
        chain_type.to_string()
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
        ChainName::Optimism | ChainName::Base | ChainName::Zora =>
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
        ChainName::Arbitrum => {
            match create_arbitrum_table_if_not_exists(table_name.clone(), &pg_client).await {
                Ok(table_result) => match table_result {
                    Some(max_blocknumber) => (max_blocknumber + 1).into(),
                    None => U64([0]),
                },
                Err(err) => panic!("Error creating table: {:?}", err),
            }
        }
    };

    let mut filter = match chain_name {
        ChainName::Optimism | ChainName::Base | ChainName::Zora => Filter::new()
            .address(address)
            .event("OutputProposed(bytes32,uint256,uint256,uint256)")
            .from_block(from_block_num)
            .to_block(new_block_num),
        ChainName::Arbitrum => Filter::new()
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
            match chain_name {
                ChainName::Optimism | ChainName::Base | ChainName::Zora => {
                    let params = handle_opstack_events(log);
                    // Insert the data into PostgreSQL
                    if let Err(err) =
                        insert_into_postgres(table_name.clone(), &pg_client, params).await
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
