# Morpheum CLI

Thin wrapper around `morpheum-sdk` providing a user-friendly command-line interface
for interacting with the Morpheum blockchain and its cross-chain infrastructure.

## Quick Start

```bash
# Build with cross-chain support
cargo build --features bank

# Add a key from BIP-39 mnemonic
morpheum keys add default --mnemonic "abandon abandon ..."

# Import a raw EVM private key
morpheum keys import-evm my-evm-wallet --private-key 0x...
```

## Cross-Chain Transfers

Deposit tokens from external chains into Morpheum and withdraw back via
Hyperlane Warp Routes.

### Deposit (External Chain → Morpheum)

```bash
# EVM: Deposit 100 USDC from Sepolia
morpheum tx bank deposit --chain evm:sepolia --token USDC --amount 100

# EVM: Deposit 0.01 ETH from Sepolia with custom RPC
morpheum tx bank deposit --chain evm:sepolia --token ETH --amount 0.01 \
  --chain-rpc https://my-rpc.example.com

# SVM: Deposit 0.5 SOL from Solana Devnet
morpheum tx bank deposit --chain svm:devnet --token SOL --amount 0.5
```

### Withdraw (Morpheum → External Chain)

```bash
# Withdraw USDC to EVM address
morpheum tx bank withdraw --chain evm:sepolia --token USDC \
  --recipient 0x000000000000000000000000<20-byte-address> --amount 1000000

# Withdraw SOL to Solana address
morpheum tx bank withdraw --chain svm:devnet --token SOL \
  --recipient 0x<32-byte-solana-pubkey> --amount 50000000
```

### Query Delivery Status

```bash
morpheum query gmp delivery --message-id 0xabc123...
```

## Architecture

The CLI follows a layered design:

- **`morpheum-cli`** — Thin command dispatcher; no business logic
- **`morpheum-sdk`** — All heavy logic (chain registries, signing, transaction building)
- **`morpheum-signing`** — BIP-39 key derivation for Native, EVM, and Solana

Cross-chain capabilities are a composable trait applied to modules, not a
separate top-level concept. The `xchain` module provides `CrossChainExecutor`
which any module can use. Currently the `bank` module uses it for
deposit/withdraw.

## Features

| Feature | Description |
|---------|-------------|
| `bank`  | Enables cross-chain deposit/withdraw via Hyperlane |

## Supported Chains

| Chain            | Spec                   | Tokens    |
|------------------|------------------------|-----------|
| Ethereum Sepolia | `evm:sepolia`          | USDC, ETH |
| Base Sepolia     | `evm:base-sepolia`     | USDC      |
| Polygon Amoy     | `evm:polygon-amoy`     | USDC      |
| Arbitrum Sepolia  | `evm:arbitrum-sepolia` | USDC      |
| Solana Devnet    | `svm:devnet`           | USDC, SOL |
