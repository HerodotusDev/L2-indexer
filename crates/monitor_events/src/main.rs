use config::{Config, File, FileFormat};
use dotenv::dotenv;
use ethers::prelude::*;
use eyre::Result;
use serde::Deserialize;
use std::{
    sync::Arc,
    thread,
    time::{self, Duration},
};
use tokio_postgres::NoTls;

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

#[derive(Debug, Deserialize)]
struct Networks {
    name: String,
    l1_contract: String,
    block_delay: u64,
    poll_period_sec: u64,
}

fn make(network_config: &str) -> Config {
    let config_name = format!("crates/monitor_events/networks/{}", network_config);
    Config::builder()
        .add_source(File::new(&config_name, FileFormat::Json))
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let rpc_url: &str = &std::env::var("RPC_URL").expect("RPC_URL must be set.");
    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set.");
    let network_config: &str = &std::env::var("NETWORK").expect("NETWORK must be set.");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    let config = make(network_config);
    // Deserialize the entire file as single struct
    let network: Networks = config.try_deserialize().unwrap();
    let _block_delay = network.block_delay;
    let _poll_period_sec = network.poll_period_sec;
    let table_name = network.name;

    let block_delay: U64 = U64([_block_delay]);
    let poll_period_sec: Duration = time::Duration::from_secs(_poll_period_sec);

    let address: Address = network.l1_contract.parse()?;

    let mut from_block_num = U64([0]);
    let mut new_block_num = client.get_block_number().await? - block_delay;

    // Establish a PostgreSQL connection
    let (pg_client, connection) = tokio_postgres::connect(db_url, NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    match create_table_if_not_exists(table_name.clone(), &pg_client).await {
        Ok(table_result) => match table_result {
            Some(max_blocknumber) => from_block_num = (max_blocknumber + 1).into(),
            _ => {}
        },
        Err(err) => eprintln!("Error creating table: {:?}", err),
    }
    let mut filter = Filter::new()
        .address(address)
        .event("OutputProposed(bytes32,uint256,uint256,uint256)")
        .from_block(from_block_num)
        .to_block(new_block_num);
    loop {
        let logs = client.get_logs(&filter).await?;
        println!(
            "from {from_block_num:?} to {new_block_num:?}, {} pools found!",
            logs.iter().len()
        );

        for log in logs.iter() {
            let l2_output_root = Bytes::from(log.topics[1].as_bytes().to_vec());
            let l2_output_index = U256::from_big_endian(&log.topics[2].as_bytes());
            let l2_block_number = U256::from_big_endian(&log.topics[3].as_bytes());
            let l1_timestamp = U256::from_big_endian(&log.data[..]);
            let l1_transaction_hash =
                Bytes::from(log.transaction_hash.unwrap().as_bytes().to_vec());
            let l1_block_number = log.block_number.unwrap();
            let l1_transaction_index = log.transaction_index.unwrap();
            let l1_block_hash = Bytes::from(log.block_hash.unwrap().as_bytes().to_vec());
            println!(
                "output_root = {l2_output_root}, l2OutputIndex = {l2_output_index}, l2BlockNumber = {l2_block_number}, l1Blocknumber = {l1_block_number}, l1Timestamp = {l1_timestamp}, l1_transaction_hash={l1_transaction_hash}, l1_transaction_index={l1_transaction_index}, L1_block_hash={l1_block_hash}",
            );

            // Insert the data into PostgreSQL
            if let Err(err) = insert_into_postgres(
                table_name.clone(),
                &pg_client,
                l2_output_root,
                l2_output_index,
                l2_block_number,
                l1_timestamp,
                l1_transaction_hash,
                l1_block_number,
                l1_transaction_index,
                l1_block_hash,
            )
            .await
            {
                eprintln!("Error inserting data into PostgreSQL: {:?}", err);
            }
        }
        thread::sleep(poll_period_sec);

        from_block_num = new_block_num + 1;
        new_block_num = client.get_block_number().await? - block_delay;
        filter = filter.from_block(from_block_num).to_block(new_block_num);
    }
}

async fn create_table_if_not_exists(
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
        return Ok(Some(max_blocknum));
    } else {
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} ( 
            id              SERIAL PRIMARY KEY,
            l2_output_root     VARCHAR NOT NULL,
            l2_output_index INTEGER NOT NULL,
            l2_blocknumber  INTEGER NOT NULL,
            l1_timestamp    INTEGER NOT NULL,
            l1_transaction_hash    VARCHAR NOT NULL,
            l1_block_number    INTEGER NOT NULL,
            l1_transaction_index    INTEGER NOT NULL,
            l1_block_hash     VARCHAR NOT NULL
        )",
            table_name
        );
        client.execute(&create_table_query, &[]).await?;

        return Ok(None);
    }
}

async fn insert_into_postgres(
    table_name: String,
    client: &tokio_postgres::Client,
    l2_output_root: Bytes,
    l2_output_index: U256,
    l2_block_number: U256,
    l1_timestamp: U256,
    l1_transaction_hash: Bytes,
    l1_block_number: U64,
    l1_transaction_index: U64,
    l1_block_hash: Bytes,
) -> Result<(), tokio_postgres::Error> {
    let insert_query = format!("INSERT INTO {} (l2_output_root, l2_output_index, l2_blocknumber, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)", table_name);
    client
        .execute(
            &insert_query,
            &[
                &l2_output_root.to_string(),
                &(l2_output_index.as_u64() as i32),
                &(l2_block_number.as_u64() as i32),
                &(l1_timestamp.as_u64() as i32),
                &l1_transaction_hash.to_string(),
                &(l1_block_number.as_u64() as i32),
                &(l1_transaction_index.as_u64() as i32),
                &l1_block_hash.to_string(),
            ],
        )
        .await?;

    Ok(())
}
