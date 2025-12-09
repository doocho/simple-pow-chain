# Simple PoW Chain

A simple Bitcoin-like Proof-of-Work blockchain implementation in Rust.

## Features

- **Proof of Work**: SHA-256 hashing with configurable difficulty
- **Longest Chain Rule**: Nodes sync to the longest valid chain
- **Coinbase Transactions**: Mining rewards (50 coins per block)
- **Transaction Signing**: ECDSA signatures with secp256k1
- **P2P Networking**: TCP-based peer-to-peer communication

## Build

```bash
cargo build --release
```

## Usage

### Start a Genesis Node

```bash
cargo run -- --genesis --mine
```

### Connect a Second Node

```bash
cargo run -- --port 8081 --peer 127.0.0.1:8080 --mine
```

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --port <PORT>` | Listen port | 8080 |
| `-e, --peer <PEER>` | Peer address (e.g., 127.0.0.1:8080) | - |
| `--genesis` | Create genesis block | false |
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
└── node.rs        # P2P node (sync, mining, broadcast)
```

## Example

Run a 2-node network:

```bash
# Terminal 1: Genesis node
cargo run -- --genesis --mine --difficulty 3

# Terminal 2: Second node
cargo run -- --port 8081 --peer 127.0.0.1:8080 --mine --difficulty 3
```

## License

MIT
