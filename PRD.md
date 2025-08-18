## PRD: Simple Proof-of-Work (PoW) Blockchain

### 1) Overview
- **Purpose**: Specify a minimal, educational blockchain that demonstrates PoW consensus, basic transactions, networking, and persistence.
- **Form factor**: CLI node and lightweight HTTP API suitable for local/dev networks; single binary.
- **Audience**: Learners, tinkerers, and developers exploring blockchain building blocks.

### 2) Goals
- **Core chain**: Append-only chain with verifiable blocks and a deterministic longest-chain rule (cumulative work).
- **PoW consensus**: Adjustable difficulty, target block time, and safe block validation.
- **Transactions**: Signed transfers and coinbase rewards with a simple balance model.
- **Networking**: Peer discovery (static list), gossip for blocks/transactions, fork resolution.
- **Persistence**: Durable storage and restart recovery.
- **Operability**: Simple CLI and minimal HTTP API; observability via logs/metrics.
- **Security (basic)**: Signature verification, replay/double-spend prevention, basic input validation.

### 3) Non-Goals
- No smart contracts/VM, no mempool prioritization/fees market, no light clients, no slashing, no advanced p2p (NAT traversal, Kademlia), no byzantine network hardening, no production-grade security.

### 4) Key Personas
- **Node Operator**: Runs a node, mines, checks status.
- **Developer**: Integrates with the HTTP API/CLI for demos/tests.
- **Learner**: Reads code, experiments with parameters.

### 5) Scope and MVP
- **In**: Blocks, PoW mining, signatures, coinbase reward, simple transfer transactions, basic p2p, persistence, CLI + minimal HTTP.
- **Out**: Smart contracts, complex fee markets, advanced consensus, complex wallet UX, mobile/GUI.

### 6) Architecture
- **Modules**:
  - Blockchain: block assembly, validation, chain selection.
  - Consensus: PoW target, difficulty adjustment, mining loop.
  - Tx Pool: mempool, tx validation, selection for block templates.
  - Networking: peer manager, gossip, block/tx propagation, simple sync.
  - State: account balances (account-based), coinbase issuance, replay protection.
  - Storage: on-disk blocks and state snapshot.
  - API/CLI: node ops, wallet ops, queries.
  - Wallet: key generation, signing, address derivation.
- **Data flow**:
  - Incoming tx → validate → mempool → miner builds block template → PoW → broadcast → peers validate → adopt if higher cumulative work.
- **Language/Stack (recommended)**: Rust, `serde`, `sha2`, `secp256k1`, `bincode`/`serde_json`, `tokio`, `reqwest`/`hyper`, `sled`/`rocksdb` (or append-only file for MVP).

### 7) Data Model
- **BlockHeader**: version, parent_hash, merkle_root_or_tx_root, timestamp, nonce, difficulty_target (compact), height.
- **Block**: header, transactions[], coinbase_tx.
- **Transaction**: from_address, to_address, amount, fee (optional minimal), nonce (per account), signature, tx_hash.
- **Account State**: map address → balance, address → next_nonce.
- **Peer**: id, address, last_seen.
- **Chain Metadata**: tip_hash, height, cumulative_work.

### 8) Consensus: PoW
- **Hash function**: SHA-256 (single or double; pick one and keep it consistent).
- **Target**: Compact representation (bits) → full target; valid if hash(header) ≤ target.
- **Difficulty**:
  - Target block time: 10 seconds (configurable).
  - Adjustment interval: every 100 blocks (configurable).
  - Bounds: limit per-adjustment change to ×/÷4 to avoid oscillations.
- **Chain selection**: Highest cumulative work (sum of difficulty for each block), not merely height.
- **Timestamp drift**: Reject blocks with timestamps more than +2 minutes from local clock.

### 9) Validation Rules
- **Block**:
  - Header hash meets target; parent exists; height = parent.height + 1.
  - Timestamp monotonic > parent; within allowed drift.
  - Coinbase reward exactly equals subsidy + total fees; coinbase has no signer.
  - Tx merkle/root matches transactions (if implemented); otherwise deterministic tx hashing list root.
- **Transaction**:
  - Signature valid for `from_address`.
  - Sufficient balance: amount + fee ≤ balance.
  - Nonce matches account’s next_nonce; increment on apply.
  - Amount > 0; fee ≥ 0; size and field bounds.
- **State transition**:
  - Apply txs in order; drop invalids when mining; reject block if any included tx invalid against parent state.

### 10) Mining
- **Template**: Build from tip; include coinbase and selected mempool txs (up to block size cap).
- **Loop**: Iterate nonce; update timestamp periodically; on new tip, rebuild template.
- **Stop conditions**: Found valid nonce or a higher-work chain tip arrives.
- **Rewards**: Fixed subsidy per block (e.g., 50 units), halving not required (configurable later).

### 11) Networking
- **Transport**: TCP or HTTP-based gossip for MVP.
- **Discovery**: Static bootstrap peers via config; manual add via API.
- **Messages**:
  - NewTx, NewBlock announcements.
  - GetHeaders(from_hash, max), Headers(list).
  - GetBlock(hash), Block(data).
- **Sync**:
  - On startup: request headers from peers, locate fork point, fetch missing blocks, validate, adopt best chain.
  - During run: handle competing forks; switch to higher cumulative work.

### 12) Persistence
- **Storage**:
  - Blocks: by hash; headers and bodies separately for speed.
  - Chain index: tip, height, cumulative_work.
  - State: snapshot at every N blocks + journal of deltas or reapply from genesis for MVP.
  - Mempool: in-memory; rebuild on startup by discarding.
- **Durability**: fsync on block commit; crash-safe enough for dev.

### 13) CLI
- **Commands**:
  - `init` (create genesis, keys, config)
  - `run` (start node: p2p, api, miner optional)
  - `mine --on|--off` (toggle mining at runtime)
  - `wallet new|show|import`
  - `send --to <addr> --amount <n> [--fee <n>]`
  - `balance [--address <addr>]`
  - `status` (height, tip, peers, mempool size, hashrate)
  - `validate` (full chain verification)
- Example:
```bash
powchain init --network local && powchain run --mine --http :8080
```

### 14) HTTP API (minimal)
- `GET /health` → {status}
- `GET /chain/tip` → {height, hash, difficulty, cumulativeWork}
- `GET /blocks/{hash}` → block
- `POST /tx` → {txHex/json} → {accepted: bool, txHash}
- `GET /address/{addr}/balance` → {balance, nonce}
- `GET /peers` / `POST /peers` → list/add peers

### 15) Configuration
- `config.toml`:
  - network_id, data_dir, http_addr, p2p_addr
  - mining: enabled, threads, target_block_time, adjustment_interval
  - consensus: max_clock_drift_secs, block_size_limit_bytes
  - peers: bootstrap[]

### 16) Observability
- **Logging**: levels, structured fields (module, height, hash, peer_id).
- **Metrics**: blocks_mined, blocks_validated, orphan_blocks, mempool_size, peer_count, avg_block_time.
- **Tracing**: optional span IDs for validation/mining.

### 17) Security (baseline)
- Input sanitization, DoS guards (size limits, rate limits on API).
- Signature verification with canonical encoding.
- Reject duplicate txs/blocks; ignore peers flooding invalid data.
- Optional peer allowlist for local demos.

### 18) Performance Targets (dev)
- Single node: ≥ 50 tx/s validation on commodity laptop.
- Mining: sustained hashrate with multithreaded nonce search.
- Sync: ≤ 5s to sync 1,000 tiny blocks over loopback.

### 19) Testing & Acceptance Criteria
- **Unit**: hashing, target expansion, signature check, tx validity, block validity, difficulty retarget math.
- **Integration**:
  - Start 3 nodes; mine blocks; tips converge on same chain.
  - Broadcast tx; included in block within 3 blocks.
  - Fork test: create competing chains; all nodes adopt higher cumulative work.
  - Restart: node resumes with correct tip and state.
- **CLI E2E**:
  - `wallet new`, `send`, `balance` reflect expected balances.
  - `validate` reports a consistent chain.
- **API**:
  - `POST /tx` accepts valid tx and rejects invalid signatures/nonces.
  - `GET /chain/tip` updates after mining a block.

### 20) Milestones
- M1: Data models, hashing, genesis, basic validation.
- M2: Wallet/keys, tx creation/verification, account state.
- M3: Mining loop, coinbase, fixed difficulty.
- M4: Difficulty adjustment, cumulative work selection.
- M5: Persistence (blocks/state), restart reliability.
- M6: Networking (gossip + sync), conflict resolution.
- M7: CLI + HTTP API, logging/metrics, docs/sample scripts.

### 21) Risks & Mitigations
- **Clock drift**: enforce drift bounds; prefer parent timestamp + 1 where needed.
- **Peer abuse**: rate limit, size caps, disconnect on repeated invalid data.
- **State replay**: snapshot + checksum; fallback to re-derive from genesis.
- **Mining starvation**: retarget floor/ceiling; auto-adjust threads.

### 22) Deliverables
- Binary with CLI and HTTP server.
- Config file template and example network bootstrap file.
- README with quickstart, API reference, and diagrams.
- Test suite with unit/integration tests and reproducible local network script.

### 23) Future Extensions (out of scope)
- Fees and mempool prioritization, UTXO model, headers-first sync, SPV client, NAT traversal and discovery, wallet encryption, pruning, RPC auth.


