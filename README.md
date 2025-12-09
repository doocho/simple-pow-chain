# Simple PoW Chain

A simple Bitcoin-like Proof-of-Work blockchain implementation in Rust.

## Features

- **Proof of Work**: SHA-256 hashing with configurable difficulty
- **Longest Chain Rule**: Nodes sync to the longest valid chain
- **Coinbase Transactions**: Mining rewards (50 coins per block)
- **Transaction Signing**: ECDSA signatures with secp256k1
- **P2P Networking**: TCP-based peer-to-peer communication
- **Seed Node**: Automatic peer discovery

## Build

```bash
cargo build --release
```

## Usage

### Using Seed Node (Recommended)

```bash
# Terminal 1: Start seed node
cargo run -- seed

# Terminal 2: First node (creates genesis automatically)
cargo run -- node --seed 127.0.0.1:9000 --mine

# Terminal 3: Second node (syncs from first node via seed)
cargo run -- node --port 8081 --seed 127.0.0.1:9000 --mine

# Terminal 4: Third node
cargo run -- node --port 8082 --seed 127.0.0.1:9000 --mine
```

### Direct Peer Connection

```bash
# Terminal 1: First node
cargo run -- node --mine

# Terminal 2: Connect directly to peer
cargo run -- node --port 8081 --peer 127.0.0.1:8080 --mine
```

## Commands

### `seed` - Run a seed node

```bash
cargo run -- seed [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --port <PORT>` | Listen port | 9000 |

### `node` - Run a blockchain node

```bash
cargo run -- node [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --port <PORT>` | Listen port | 8080 |
| `-s, --seed <ADDR>` | Seed node address for peer discovery | - |
| `-e, --peer <ADDR>` | Direct peer address | - |
| `-d, --difficulty <N>` | PoW difficulty (leading zeros) | 4 |
| `-m, --miner <ADDR>` | Miner address for rewards | miner |
| `--mine` | Enable auto-mining | false |

## Architecture

```
src/
├── main.rs        # CLI entry point
├── block.rs       # Block structure with PoW mining
├── blockchain.rs  # Chain management and validation
├── transaction.rs # Transactions with ECDSA signing
├── message.rs     # P2P network message types
├── node.rs        # P2P node (sync, mining, broadcast)
└── seed.rs        # Seed node for peer discovery
```

## How It Works

1. **Seed Node**: Maintains a registry of active peers. New nodes register with the seed and receive a list of known peers.

2. **Node Discovery**: When a node starts with `--seed`, it:
   - Registers itself with the seed node
   - Gets list of other registered peers
   - Connects to discovered peers

3. **Blockchain Sync**: Nodes try to sync from peers. If no peers available, genesis block is created automatically.

4. **Mining**: Nodes with `--mine` continuously mine new blocks and broadcast them to peers.

## License

MIT
