# mempool-history

[![MIT License](https://img.shields.io/github/license/rjected/mempool-history)](https://github.com/rjected/mempool-history/blob/main/LICENSE)
[![Test and Lint](https://github.com/Rjected/mempool-history/actions/workflows/ci.yml/badge.svg)](https://github.com/Rjected/mempool-history/actions/workflows/ci.yml)

This tool uses ethereum pending transaction RPCs to listen for mempool transactions and timestamp them.

The purpose of this tool is to gather data that roughly matches the "true" mempool history so it can be easily replayed while testing other applications.

## Requirements
 * Rust
 * A **websocket** RPC endpoint with a working `eth_subscribe` and `eth_getTransactionByHash` endpoint

## Usage
For now, this is how you run the tool:
```bash
cargo run -- --rpc-url <YOUR_WEBSOCKET_URL>
```

## TODO
 - [ ] Persist timestamped transactions
   - [ ] Figure out how to gracefully recover after a crash. Make sure the tool does not overwrite timestamps of a transaction that already exists. (_stretch goal_)
 - [ ] Simplify if on alchemy by using `alchemy_pendingTransactions`
 - [ ] Prettier or more useful output (JSON? frontend?) (_stretch goal_)
   - [ ] Prioritize showing `Transaction`s that are not yet included in a block
   - [ ] Relate unconfirmed transactions with consecutive account nonces
   - [ ] [mempool.space](https://mempool.space) but for ethereum? (_extreme stretch goal_)
 - [ ] Name the tool
 - [ ] Add tests
 - [ ] Turn these bullet points into github issues
