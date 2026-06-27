# Changelog

## [0.12.2] – 2026-01-20

### Added

* feat: add new `get_address_utxos` method [#134]
* feat: add new `Utxo` and `UtxoStatus` API types [#134]
* feat: add justfile [#140]
* feat(api): add `ScriptHashTxsSummary` and `ScriptHashStats` structs [#143]
* feat(api): add `BlockInfo` struct [#143]
* feat(api): add `MempoolStats` struct [#143]
* feat(api): add `MempoolRecentTx` struct [#143]
* feat(client): add `get_tx_outspends` method (`GET /tx/:txid/outspends`) [#143]
* feat(client): add `get_scripthash_stats` method (`GET /scripthash/:hash`) [#143]
* feat(client): add `get_mempool_address_txs` method (`GET /address/:address/txs/mempool`) [#143]
* feat(client): add `get_mempool_scripthash_txs` method (`GET /scripthash/:hash/txs/mempool`) [#143]
* feat(client): add `get_scripthash_utxos` method (`GET /scripthash/:hash/utxo`) [#143]
* feat(client): add `get_block_info` method (`GET /block/:hash`) [#143]
* feat(client): add `get_block_txids` method (`GET /block/:hash/txids`) [#143]
* feat(client): add `get_block_txs` method (`GET /block/:hash/txs[/:start_index]`) [#143]
* feat(client): add `get_mempool_stats` method (`GET /mempool`) [#143]
* feat(client): add `get_mempool_txids` method (`GET /mempool/txids`) [#143]
* feat(client): add `get_mempool_recent_txs` method (`GET /mempool/recent`) [#143]
* chore(docs): add missing documentation [#147]
* feat(client): add new `submit_package` API to `BlockingClient` and `AsyncClient` [#114]
* feat(api): add new `SubmitPackageResult`, `TxResult`, and `MempoolFeesSubmitPackage` API structures [#114]

### Changed

* fix(ci): pin dependencies to MSRV supported versions [#138]
* chore(deps): bump webpki-roots to 1.0.4, pin quote to 1.0.41 [#139]
* feat(ci): always run CI workflow [#144]
* fix(ci): bump pinned webpki-roots to 1.0.5 and pin other dependencies [#153]
* feat(client): update the `post_request_hex` method to `post_request_bytes`, now handling `query_params` and having `Response` as return type [#114]
* feat(client): update the internals of the  `broadcast` method to use new `post_request` and `post_request_bytes`, with no breaking change [#114]
* chore(submit_package): use `unwrap_or_default` instead of `.unwrap()` [#159]


[0.12.2]: https://github.com/bitcoindevkit/rust-esplora-client/compare/v0.12.1...v0.12.2
