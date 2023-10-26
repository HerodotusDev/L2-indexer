# Monitor Events

**Run monitoring service that gets network spec as pm2 config. It keeps track OP Stack rollup that emitted block hash preimage(Output Root) on the L1 contract.**

### Quick Start with PM2

```sh
cargo install
```

```sh
cargo build --release
```

You need to modify the `pm2.config.js` file to run monitoring services in multiple processes in one command.

```sh
pm2 start pm2.config.js
```

This will launch 6 Apps to monitor events on Optimism, Zora, and Base for Goerli testnet and Mainnet respectively. If you want your own OP Stack modify the file.

```
[PM2] App [monitor-base-goerli] launched (1 instances)
[PM2] App [monitor-zora-goerli] launched (1 instances)
[PM2] App [monitor-optimism-goerli] launched (1 instances)
[PM2] App [monitor-base-mainnet] launched (1 instances)
[PM2] App [monitor-optimism-mainnet] launched (1 instances)
[PM2] App [monitor-zora-mainnet] launched (1 instances)
```

Don't forget to update the `.env` file. You need DB_URL for database connection, you need RPC_URL for query event from contract.

Also, you need to put NETWORK for the config network you want to monitor.

```json
{
  // It will be the table name of your postsql
  "name": "base",
  // You need to get L1 contract the OPstack chain sends transactions to settle.
  "l1_contract": "0x56315b90c40730925ec5485cf004d835058518A0",
  // You can customize your own block delay number. It will wait monitoring service to get a more finalized block.
  "block_delay": 20,
  // After you run the service, it will poll the event emitted again after the second below.
  "poll_period_sec": 60,
  // (Optional) eth_getLogat have rate limit. So especially when calling like base_goerli or optimism_goerli, if you don't batch the request, will face an error. If you don't put any parameter default will be the latest block number.
  "batch_size": 100000
}
```

First, you need to run a monitoring service. It will start monitoring events from L1 contract and store output roots in the database. You can run it with:

```sh
cargo run -p monitor_events
```
