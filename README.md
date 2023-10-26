# OP Stack Indexer

**This repo implements an indexer who's role is to watch for OP Stack events emitted on Ethereum and index the Output Roots into a database for querying later -- providing example for Optimism, Base, Zora chain**

### Architecture

1. **[Monitor Events](/crates/monitor_events/README.md)**: Run monitoring service that gets network spec as pm2 config. It keeps track OP Stack rollup that emitted block hash preimage(Output Root) on the L1 contract.
2. **[OP Stack Micro Service](/crates/opstack_ms/README.md)**: Rust HTTP micro-service provides OP Stack information including block hash requested by block number.

### Monitoring service:

Monitor events (`OutputProposed`) from L1 contract. Retrieve `output_root`

First check table is exist. If it's exist, get latest L1 block that stored in db, and use this block number as `from_block` filter. If it's not exist, create table and query all events from `0` to `latest block - BLOCK_DELAY`.

Monitor service will pull every `POLL_PERIOD`. If `POLL_PERIOD` is longer than block creation time, the service algorithm is already ensure to get not duplicated event. Here is the example log:

example log

```sh
output_root = 0x43949a1178f9fbcd851c5f6103603d7f7df0c05e399d09c7edb96ef4281a9d25, l2OutputIndex = 2873, l2BlockNumber = 110408263, l1Blocknumber = 18276691, l1Timestamp = 1696416911, l1_transaction_hash=0xbf90fd89af4a580695abd69bccce1ed3ef426e72021ee3c7e0aad2f4b3d8375d, l1_transaction_index=195, L1_block_hash=0x3d05fd1575b8b38b08a1e8d2a4253b09fba7e01f72e66e8c19eec0a3b39bc62f
from 18276743 to 18276747, 0 pools found!
from 18276748 to 18276752, 0 pools found!
from 18276753 to 18276757, 0 pools found!
```

### Block number -> Output Index mapper Microservice

Request with `l2_block_number`, return `l2_output_root` and `l2_block_number`. In this case, we can query the database directly:

```sql
  SELECT l2_output_root, l2_output_index, l2_blocknumber, l1_timestamp, l1_transaction_hash, l1_block_number, l1_transaction_index, l1_block_hash
            FROM optimism
            WHERE l2_blocknumber >= $1
            ORDER BY l2_blocknumber ASC
            LIMIT 1;
```

So that it return the nearest, but newer blocknumber from what was requested.

Endpoint

```
http://127.0.0.1:8000/output-root
```
