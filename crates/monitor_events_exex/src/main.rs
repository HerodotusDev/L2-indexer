use std::{str::FromStr, time::Duration};

use alloy_sol_types::{sol, SolEventInterface};
use arbitrum::create_arbitrum_table_if_not_exists;
use common::{get_network_config, ChainName, ChainType};
use dotenv::dotenv;
use futures::Future;
use opstack::create_opstack_table_if_not_exists;
use reth_exex::{ExExContext, ExExEvent};
use reth_node_api::FullNodeComponents;
use reth_node_ethereum::EthereumNode;
use reth_primitives::{Address, Log, SealedBlockWithSenders, TransactionSigned, U256, U64};
use reth_provider::Chain;
use reth_tracing::tracing::info;
use tokio::time;
use tokio_postgres::{Client, NoTls};

pub mod arbitrum;
pub mod opstack;

sol!(L2OutputOracle, "opstack_abi.json");
use crate::L2OutputOracle::{L2OutputOracleEvents, OutputProposed};

/// Initializes the ExEx.
///
/// Opens up a SQLite database and creates the tables (if they don't exist).
async fn init<Node: FullNodeComponents>(
    ctx: ExExContext<Node>,
    chain_name: ChainName,
    table_name: String,
    deployment_block: U64,
    pg_client: Client,
) -> eyre::Result<impl Future<Output = eyre::Result<()>>> {
    let mut from_block_num = match chain_name {
        ChainName::Optimism | ChainName::Base | ChainName::Zora => {
            create_opstack_table_if_not_exists(table_name.clone(), &pg_client).await
        }
        ChainName::Arbitrum => {
            create_arbitrum_table_if_not_exists(table_name.clone(), &pg_client).await
        }
    }
    .expect("Error creating table")
    .map_or(deployment_block, |max_blocknumber| {
        U64::from(max_blocknumber + 1)
    });

    Ok(l2_indexer_exex(ctx, table_name, pg_client))
}

async fn l2_indexer_exex<Node: FullNodeComponents>(
    mut ctx: ExExContext<Node>,
    table_name: String,
    mut client: Client,
) -> eyre::Result<()> {
    // Process all new chain state notifications
    while let Some(notification) = ctx.notifications.recv().await {
        // Revert all deposits and withdrawals
        if let Some(reverted_chain) = notification.reverted_chain() {
            let events = decode_chain_into_events(&reverted_chain);

            let mut deposits = 0;

            for (_, tx, _, event) in events {
                match event {
                    // output proposed event
                    L2OutputOracleEvents::OutputProposed(OutputProposed {
                        outputRoot,
                        l2OutputIndex,
                        l2BlockNumber,
                        l1Timestamp,
                    }) => {
                        let delete_query = format!(
                            "DELETE FROM {} WHERE l2_output_root = $1 AND l2_output_index = $2 AND l2_block_number = $3 AND l1_timestamp = $4",
                            table_name
                        );
                        let l2_output_index: i32 = l2OutputIndex.try_into().unwrap();
                        let l2_block_number: i32 = l2BlockNumber.try_into().unwrap();
                        let l1_timestamp: i32 = l1Timestamp.try_into().unwrap();
                        let deleted = client
                            .execute(
                                &delete_query,
                                &[
                                    &outputRoot.to_string(),
                                    &l2_output_index,
                                    &l2_block_number,
                                    &l1_timestamp,
                                ],
                            )
                            .await?;
                        deposits += deleted;
                    }
                    _ => continue,
                }
            }

            info!(block_range = ?reverted_chain.range(), %deposits, "Reverted chain events");
        }

        // Insert all new deposits and withdrawals
        if let Some(committed_chain) = notification.committed_chain() {
            let events = decode_chain_into_events(&committed_chain);

            let mut root_commit = 0;

            for (block, tx, log, event) in events {
                match event {
                    L2OutputOracleEvents::OutputProposed(OutputProposed {
                        outputRoot,
                        l2OutputIndex,
                        l2BlockNumber,
                        l1Timestamp,
                    }) => {
                        // iterate all txs and get index that matches the tx
                        // TODO: i don't think this is best
                        let tx_index = block
                            .transactions()
                            .position(|tx_in_block| tx_in_block.hash == tx.hash)
                            .unwrap();

                        let insert_query = format!("INSERT INTO {} (l2_output_root, l2_output_index, l2_block_number, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)", table_name);
                        let l2_output_index: i32 = l2OutputIndex.try_into().unwrap();
                        let l2_block_number: i32 = l2BlockNumber.try_into().unwrap();
                        let l1_timestamp: i32 = l1Timestamp.try_into().unwrap();
                        let inserted = client
                            .execute(
                                &insert_query,
                                &[
                                    &outputRoot.to_string(),
                                    &l2_output_index,
                                    &l2_block_number,
                                    &l1_timestamp,
                                    &tx.hash.to_string(),
                                    &(block.block.number as i32),
                                    &(tx_index as i32),
                                    &block.block.hash().to_string(),
                                ],
                            )
                            .await?;
                        root_commit += inserted;
                    }
                    _ => continue,
                };
            }

            info!(block_range = ?committed_chain.range(), %root_commit, "Committed chain events");

            // Send a finished height event, signaling the node that we don't need any blocks below
            // this height anymore
            ctx.events
                .send(ExExEvent::FinishedHeight(committed_chain.tip().number))?;
        }
    }

    Ok(())
}

/// Decode chain of blocks into a flattened list of receipt logs, and filter only
/// [L2OutputOracleEvents].
fn decode_chain_into_events(
    chain: &Chain,
) -> impl Iterator<
    Item = (
        &SealedBlockWithSenders,
        &TransactionSigned,
        &Log,
        L2OutputOracleEvents,
    ),
> {
    chain
        // Get all blocks and receipts
        .blocks_and_receipts()
        // Get all receipts
        .flat_map(|(block, receipts)| {
            block
                .body
                .iter()
                .zip(receipts.iter().flatten())
                .map(move |(tx, receipt)| (block, tx, receipt))
        })
        // Get all logs
        .flat_map(|(block, tx, receipt)| receipt.logs.iter().map(move |log| (block, tx, log)))
        // Decode and filter bridge events
        .filter_map(|(block, tx, log)| {
            L2OutputOracleEvents::decode_raw_log(log.topics(), &log.data.data, true)
                .ok()
                .map(|event| (block, tx, log, event))
        })
}

fn main() -> eyre::Result<()> {
    // Settup the environment variables
    dotenv().ok();
    let chain_type: ChainType =
        ChainType::from_str(&std::env::var("CHAIN_TYPE").expect("TYPE must be set.")).unwrap();
    let chain_name: ChainName =
        ChainName::from_str(&std::env::var("CHAIN_NAME").expect("TYPE must be set.")).unwrap();
    let network = get_network_config(chain_type, chain_name);
    let block_delay = network.block_delay;
    let poll_period_sec = network.poll_period_sec;
    let table_name = network.name;
    let block_delay: U64 = U64::from(block_delay);
    let poll_period_sec: Duration = time::Duration::from_secs(poll_period_sec);
    let address: Address = network.l1_contract.parse()?;
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let handle = builder
            .node(EthereumNode::default())
            .install_exex("L2Indexer", move |ctx| async move {
                let db_url: &str = &std::env::var("DB_URL").expect("DB_URL must be set.");
                // Establish a PostgreSQL connection
                let (pg_client, connection) = tokio_postgres::connect(db_url, NoTls)
                    .await
                    .expect("Failed to connect to PostgreSQL");

                init(
                    ctx,
                    chain_name,
                    table_name,
                    U64::from(network.l1_contract_deployment_block),
                    pg_client,
                )
                .await
            })
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
}
