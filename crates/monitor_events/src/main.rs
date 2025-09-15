use ::common::{get_network_config, Network, ChainName, ChainType};
use arbitrum::create_arbitrum_table_if_not_exists;
use dotenv::dotenv;
use ethers::prelude::*;
use eyre::Result;
use opstack::create_opstack_table_if_not_exists;
use opstack::create_opstack_dispute_games_table_if_not_exists;
use opstack::get_highest_game_index;

use std::{
    str::FromStr,
    sync::Arc,
    thread,
    time::{self, Duration},
};
use tokio_postgres::NoTls;

use crate::{
    arbitrum::handle_arbitrum_events,
    opstack::handle_opstack_events,
    opstack::handle_opstack_fdg_events
};

mod arbitrum;
mod fetcher;
mod opstack;

#[tokio::main]
async fn main() -> Result<()> {
    // CRITICAL: This indexer is designed to panic and restart on any event handling
    // or database insert failures to ensure data integrity and prevent broken index counting.
    // This is intentional behavior - do not suppress these panics.
    
    // Settup the environment variables
    dotenv().ok();
    let rpc_url: &str = &std::env::var("RPC_URL").expect("RPC_URL must be set.");
    let chain_name_str = std::env::var("CHAIN_NAME").expect("CHAIN_NAME must be set.");
    let chain_type_str = std::env::var("CHAIN_TYPE").expect("CHAIN_TYPE must be set.");

    let chain_name = ChainName::from_str(&chain_name_str)
        .expect("Invalid CHAIN_NAME");
    let chain_type = ChainType::from_str(&chain_type_str)
        .expect("Invalid CHAIN_TYPE");

    let network = Network { chain_name, chain_type };

    let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set.");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let rpc_client = Arc::new(provider);

    let network_config = get_network_config(chain_type, chain_name);
    let block_delay = network_config.block_delay;
    let poll_period_sec = network_config.poll_period_sec;
    let base_table_name = network_config.name.clone();
    let mut table_name = base_table_name.clone();
    let block_delay: U64 = U64([block_delay]);
    let poll_period_sec: Duration = time::Duration::from_secs(poll_period_sec);
    //let address: Address = network_config.l1_contract.parse()?;



    // Set block number values to filter
    // Get the latest block number and ensure we don't try to index beyond what's available
    let _latest_block = rpc_client.get_block_number().await?; // Used for initialization, but not needed after
    let mut new_block_num; // Will be initialized in the loop
    let batch_size = network_config.batch_size.unwrap_or(50000); // Use a reasonable default if not specified

    // Establish a PostgreSQL connection
    let (pg_client, connection) = tokio_postgres::connect(db_url, NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });


    let mut from_block_num_op = match chain_name {
        ChainName::Optimism | ChainName::Base | ChainName::Zora | ChainName::WorldChain => {
            create_opstack_table_if_not_exists(table_name.clone(), &pg_client).await
        }
        ChainName::Arbitrum | ChainName::ApeChain => {
            create_arbitrum_table_if_not_exists(table_name.clone(), &pg_client).await
        }
    }
    .expect("Error creating table")
    .map_or(
        U64([network_config.l1_contract_deployment_block]),
        |max_blocknumber| (max_blocknumber + 1).into(),
    );

    // Enable FDG indexing stream for Optimism Mainnet so we can backfill
    // any games that may have been created before the transition block,
    // while still retaining OutputProposed indexing before the transition.
    let fdg_enabled = chain_name == ChainName::Optimism && (
        chain_type == ChainType::Mainnet ||  chain_type == ChainType::Sepolia
    );

    // Always start from the 0th game index by default
    let mut highest_fdg_index = 0u64;
    let fault_dispute_games_table_name = format!("{}_fault_dispute_games", network_config.name);
    
    // Separate FDG from-block tracking
    let mut from_block_num_fdg: U64 = U64([0]);

    if fdg_enabled {
            println!(
                "already using FDG index mode"
            );
            // Create table if needed and get the max l1_block_number if any rows exist
            let from_block_num_fdg_opt =
                create_opstack_dispute_games_table_if_not_exists(
                    fault_dispute_games_table_name.clone(),
                    &pg_client,
                )
                .await?;                 // propagate DB error if any

            // If FDG has been indexed before, continue from the next block;
            // otherwise start from the dispute game contract deployment block
            from_block_num_fdg = match from_block_num_fdg_opt {
                Some(max_l1_block) => U64::from((max_l1_block + 1) as u64),
                None => U64([network_config.l1_dispute_game_contract_deployment_block.unwrap_or(0)]),
            };

            // Only applicable for FDG enabled L2 chains

            let highest_fdg_index_db = get_highest_game_index(&fault_dispute_games_table_name, &pg_client).await?;
            // If table has rows, continue from the next index; otherwise start at 0
            highest_fdg_index = match from_block_num_fdg_opt {
                Some(_) => highest_fdg_index_db.saturating_add(1),
                None => 0,
            };
            println!("Highest game_index: {}", highest_fdg_index);
    }

    println!(
        "starting indexing from blocks op={from_block_num_op:?}, fdg={from_block_num_fdg:?}"
    );


    // let event_signature = match chain_name {
    //     ChainName::Optimism | ChainName::Base | ChainName::Zora | ChainName::WorldChain => {
    //         "OutputProposed(bytes32,uint256,uint256,uint256)"
    //     }
    //     ChainName::Arbitrum | ChainName::ApeChain => "SendRootUpdated(bytes32,bytes32)",
    // };

    // Filter will be created and used in the event processing loops
    // let mut filter = Filter::new();

    // Loop to get the logs with time gap and with batch
    loop {
        // Get the current latest block and apply block delay to avoid indexing recent blocks
        let current_latest_block = rpc_client.get_block_number().await?;
        let safe_block_number = if current_latest_block.as_u64() > block_delay.as_u64() {
            current_latest_block - block_delay
        } else {
            U64([0]) // Fallback to 0 if block_delay is larger than current block
        };
        
        // Update new_block_num to the safe block number
        new_block_num = safe_block_number;
        
        // Ensure we don't try to index blocks that don't exist
        if new_block_num.as_u64() < from_block_num_op.as_u64() {
            println!("Waiting for more blocks to be available. Current: {}, From: {}", 
                     new_block_num, from_block_num_op);
            thread::sleep(poll_period_sec);
            continue;
        }
        
        // Compute OP upper limit range
        let block_gap_op = new_block_num.as_u64().saturating_sub(from_block_num_op.as_u64());
        let upper_limit_op = if block_gap_op > batch_size {
            from_block_num_op.as_u64() + batch_size - 1
        } else {
            new_block_num.as_u64()
        };

        // For Optimism, cap OP indexing to the transition block if provided
        let op_transition_cap = network_config
            .transition_to_dispute_game_system_block
            .unwrap_or(u64::MAX);
        let effective_upper_limit_op = upper_limit_op.min(op_transition_cap.saturating_sub(1));

        // Process OP/OutputProposed-style events
        match chain_name {
            ChainName::Optimism | ChainName::Base | ChainName::Zora | ChainName::WorldChain => {
                if from_block_num_op.as_u64() <= effective_upper_limit_op {
                    table_name = base_table_name.clone();
                    let address = network_config.l1_contract.parse::<Address>()?;
                    let filter = Filter::new()
                        .event("OutputProposed(bytes32,uint256,uint256,uint256)")
                        .address(address)
                        .from_block(from_block_num_op)
                        .to_block(U64([effective_upper_limit_op]));

                    let logs = rpc_client.get_logs(&filter).await?;
                    println!(
                        "OP events: from {from_block_num_op:?} to {effective_upper_limit_op:?}, found {}",
                        logs.iter().len()
                    );

                    for log in logs.iter() {
                        let params = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            handle_opstack_events(log)
                        })) {
                            Ok(params) => params,
                            Err(panic_info) => {
                                eprintln!("CRITICAL ERROR: handle_opstack_events panicked: {:?}", panic_info);
                                eprintln!("This indicates a serious problem that could break the indexing system. PANICKING to ensure data integrity.");
                                
                                // Panic to force restart and prevent data corruption
                                panic!("OPStack event handling panicked. This is a critical error that could break the index. Restarting to ensure data integrity. Panic info: {:?}", panic_info);
                            }
                        };
                        
                        if let Err(err) =
                            opstack::insert_into_postgres(table_name.clone(), &pg_client, params).await
                        {
                            eprintln!("CRITICAL ERROR: Failed to insert OPStack event into PostgreSQL: {:?}", err);
                            eprintln!("This indicates a serious problem with the indexing system. PANICKING to ensure data integrity.");
                            
                            // Panic to force restart and prevent data corruption
                            panic!("Database insert failed for OPStack event. This is a critical error that could corrupt the index. Restarting to ensure data integrity. Error: {:?}", err);
                        }
                    }

                    from_block_num_op = U64([effective_upper_limit_op + 1]);
                }
            }
            ChainName::Arbitrum | ChainName::ApeChain => {
                if from_block_num_op.as_u64() <= upper_limit_op {
                    table_name = base_table_name.clone();
                    let address = network_config.l1_contract.parse::<Address>()?;
                    let filter = Filter::new()
                        .event("SendRootUpdated(bytes32,bytes32)")
                        .address(address)
                        .from_block(from_block_num_op)
                        .to_block(U64([upper_limit_op]));

                    let logs = rpc_client.get_logs(&filter).await?;
                    println!(
                        "Arbitrum/ApeChain events: from {from_block_num_op:?} to {upper_limit_op:?}, found {}",
                        logs.iter().len()
                    );

                    for log in logs.iter() {
                        let params = match handle_arbitrum_events(log, &chain_name, &chain_type).await {
                            Ok(params) => params,
                            Err(err) => {
                                eprintln!("CRITICAL ERROR: Failed to handle Arbitrum event: {:?}", err);
                                eprintln!("This indicates a serious problem that could break the indexing system. PANICKING to ensure data integrity.");
                                
                                // Panic to force restart and prevent data corruption
                                panic!("Arbitrum event handling failed. This is a critical error that could break the index. Restarting to ensure data integrity. Error: {:?}", err);
                            }
                        };
                        if let Err(err) =
                            arbitrum::insert_into_postgres(table_name.clone(), &pg_client, params).await
                        {
                            eprintln!("CRITICAL ERROR: Failed to insert Arbitrum event into PostgreSQL: {:?}", err);
                            eprintln!("This indicates a serious problem with the indexing system. PANICKING to ensure data integrity.");
                            
                            // Panic to force restart and prevent data corruption
                            panic!("Database insert failed for Arbitrum event. This is a critical error that could corrupt the index. Restarting to ensure data integrity. Error: {:?}", err);
                        }
                    }

                    from_block_num_op = U64([upper_limit_op + 1]);
                }
            }
        }

        // Process FDG events for Optimism mainnet in parallel to OP events
        if fdg_enabled {
            // Ensure we don't try to index FDG blocks that don't exist
            if new_block_num.as_u64() < from_block_num_fdg.as_u64() {
                println!("Waiting for more FDG blocks to be available. Current: {}, From: {}", 
                         new_block_num, from_block_num_fdg);
            } else {
                let block_gap_fdg = new_block_num.as_u64().saturating_sub(from_block_num_fdg.as_u64());
                let upper_limit_fdg = if block_gap_fdg > batch_size {
                    from_block_num_fdg.as_u64() + batch_size - 1
                } else {
                    new_block_num.as_u64()
                };

                let factory_addr = network_config
                    .dispute_game_factory_l1_contract
                    .as_ref()
                    .expect("dispute_game_factory_l1_contract must be set")
                    .parse::<Address>()?;

                table_name = fault_dispute_games_table_name.clone();
                let filter = Filter::new()
                    .event("DisputeGameCreated(address,uint32,bytes32)")
                    .address(factory_addr)
                    .from_block(from_block_num_fdg)
                    .to_block(U64([upper_limit_fdg]));

                let logs = rpc_client.get_logs(&filter).await?;
                println!(
                    "FDG events: from {from_block_num_fdg:?} to {upper_limit_fdg:?}, created {} games",
                    logs.iter().len()
                );

                for log in logs.iter() {
                    println!("Processing FDG event, current highest_fdg_index: {}", highest_fdg_index);
                    match handle_opstack_fdg_events(log, &network, rpc_client.clone(), highest_fdg_index).await {
                        Ok(params) => {
                            println!("Successfully handled FDG event, attempting database insert...");
                            if let Err(err) = opstack::insert_fdg_into_postgres(table_name.clone(), &pg_client, params).await {
                                eprintln!("CRITICAL ERROR: PostgreSQL insert error: {err:?}");
                                eprintln!("Game index NOT incremented due to database insert failure. Current index: {}", highest_fdg_index);
                                eprintln!("This indicates a serious problem that could break index counting. PANICKING to ensure data integrity.");
                                eprintln!("STOPPING ALL PROCESSING to prevent data corruption.");
                                
                                // Panic to force restart and prevent broken index counting
                                panic!("Database insert failed for FDG event with game index {}. This is a critical error that could break index counting. Restarting to ensure data integrity. Error: {:?}", highest_fdg_index, err);
                            } else {
                                // Only increment game_index on successful database insert
                                let old_index = highest_fdg_index;
                                highest_fdg_index += 1;
                                println!("Successfully processed FDG event with game_index: {} -> {}", old_index, highest_fdg_index);
                            }
                        }
                        Err(err) => {
                            eprintln!("CRITICAL ERROR: Failed to handle DisputeGameCreated: {err:?}");
                            eprintln!("Error details: {:?}", err);
                            eprintln!("Game index NOT incremented due to event handling failure. Current index: {}", highest_fdg_index);
                            eprintln!("This indicates a serious problem that could break index counting. PANICKING to ensure data integrity.");
                            eprintln!("STOPPING ALL PROCESSING to prevent data corruption.");
                            
                            // Panic to force restart and prevent broken index counting
                            panic!("FDG event handling failed for game index {}. This is a critical error that could break index counting. Restarting to ensure data integrity. Error: {:?}", highest_fdg_index, err);
                        }
                    }
                }

                from_block_num_fdg = U64([upper_limit_fdg + 1]);
            }
        }

        // Sleep/poll update cadence
        thread::sleep(poll_period_sec);
    }
}
