# OP Stack Micro Service

**Rust HTTP micro-service provides OP Stack information including block hash requested by block number.**

### Quick Start

First, you need to run the monitoring service and your DB_URL should be full of data that got from monitoring.

Then you can run a server that exposes endpoint to request `output_root`

```sh
cargo run -p optimism_ms
```

After your Rocket has launched, you need to send `l2_block` and `network` to get `output_root` for that block:

#[post("/output_root")]

```json
{
  "name": "optimism",
  "l2_block": 105240464
}
```

Response :

```json
{
  "l2_output_root": "0x9b5482216a077163ed533a7f5a0379500f720583a07ec25e8deaa62a88aa4956",
  "l2_output_index": 3,
  "l2_blocknumber": 105242263,
  "l1_timestamp": 1686084995,
  "l1_transaction_hash": "0xbad3d21794607d1584b17a64925191aafcfc1479fb851030b3b8a11b58ec5d6b",
  "l1_block_number": 17423911,
  "l1_transaction_index": 146,
  "l1_block_hash": "0x021dcc4c09f46e1daa3ea7db4949be5da934aad91a9b07eebc05b61e048edaae"
}
```
