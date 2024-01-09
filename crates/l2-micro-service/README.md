# L2 Micro Service

**Rust HTTP micro-service provides OP Stack & Arbitrum information including block hash requested by block number.**

### Quick Start

First, you need to run the monitoring service and your DB_URL should be full of data that got from monitoring.

Then you can run a server that exposes endpoint to request `output_root`

```sh
cargo run -p l2-micro-service
```

After your Rocket has launched, you need to send `l2_block` and `network` to get `output_root` for that block, and `network` should be compatible with table name:

#[post("/output_root")]

```json
{
  "name": "optimism_mainnet",
  "l2_block": 105240464
}
```

Response :

```json
{
  "OpStack": {
    "l2_output_root": "0x051d3a95aef15113b3460d05eab6e4cb6c18d7161fcdcd1fcaa006d6293646f4",
    "l2_output_index": 1,
    "l2_blocknumber": 105238663,
    "l1_timestamp": 1686077699,
    "l1_transaction_hash": "0x8714995c6402eb33047989223371bed0f4ae2277c0e789ffe2ca38c02fcb48fd",
    "l1_block_number": 17423308,
    "l1_transaction_index": 120,
    "l1_block_hash": "0xdacff13b80de1f090ab3ffbeccbb29d92c7e1267d7b7147df8609905dcab1512"
  }
}
```

#[post("/output_root")]

```json
{
  "name": "arbitrum_mainnet",
  "l2_block": 22439717
}
```

Response :

```json
{
  "Arbitrum": {
    "l2_output_root": "0x6976d57f8aeee1758e7dadab5d140c4e2040fc4166a6eb46d43e47d0232c730b",
    "l2_block_hash": "0x568fefdabbe5b2becf087e21c377340a56b5986f07d4569fac1a27fb8c7c76f1",
    "l2_block_number": 22446774,
    "l1_transaction_hash": "0x92d5a26aa1f2728d6762730b95f67e6f80efca6df8382005b63ba96e923c18e3",
    "l1_block_number": 15503705,
    "l1_transaction_index": 104,
    "l1_block_hash": "0xc0939491c3f56c9b524a9cdda1c17663b10ab497212348e7cede9e69566f2b0d"
  }
}
```
