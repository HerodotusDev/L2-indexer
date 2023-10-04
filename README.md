# optimism_on_starknet

### Monitoring service:

Monitor events (`OutputProposed`) from L1 contract. Retrieve `output_root`

Try out

```sh
cargo run
```

First check table is exist. If it's exist, get latest L1 block that stored in db, and use this block number as `from_block` filter. If it's not exist, create table and query all events from `0` to `latest block - BLOCK_DELAY`.

Monitor service will pull every `POLL_PERIOD`. If `POLL_PERIOD` is longer than block creation time, the service algorithm is already ensure to get not duplicated event. Here is the example log:

example log

```sh
output_root = 0x43949a1178f9fbcd851c5f6103603d7f7df0c05e399d09c7edb96ef4281a9d25, l2OutputIndex = 2873, l2BlockNumber = 110408263, l1Blocknumber = 18276691, l1Timestamp = 1696416911, l1_transaction_hash=0xbf90fd89af4a580695abd69bccce1ed3ef426e72021ee3c7e0aad2f4b3d8375d, l1_transaction_index=195, L1_block_hash=0x3d05fd1575b8b38b08a1e8d2a4253b09fba7e01f72e66e8c19eec0a3b39bc62f
from 18276743 to 18276747, 0 pools found!
from 18276748 to 18276752, 0 pools found!
from 18276753 to 18276757, 0 pools found!
```

---

### Blockhash contract:
