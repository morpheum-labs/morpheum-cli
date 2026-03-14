# Morpheum CLI

Human-friendly CLI for interacting with the Morpheum platform: issue VCs, manage agents, trade markets, identity, staking, DAO, etc.

## Install

```bash
cargo install --path .
```

Or from git (when published):

```bash
cargo install --git https://github.com/morpheum-labs/morpheum-cli
```

## Quick start

```bash
# Setup wizard (stores config in ~/.morpheum, prompts for mnemonic securely)
morpheum setup

# Issue delegated VC to an agent (requires mnemonic from setup or MORPHEUM_MNEMONIC)
morpheum vc issue --agent 0xabc... --permissions TRADE --max-usd 10000 --expiry-days 90

# Dry-run (no mnemonic needed, shows planned params)
morpheum vc issue --agent 0xabc... --dry-run

# JSON output (for scripting)
morpheum --output json vc issue --agent 0xabc... --dry-run

# Create a market order (table output)
morpheum market order --market-id 42 --side buy --size 5 --price 42069
```

## Config

- Default config path: `~/.morpheum/config.toml`
- Override with `--config` or `MORPHEUM_CONFIG` env
- Mnemonic: from config file or `MORPHEUM_MNEMONIC` env (env overrides file; never commit!)

## Output formats

- `--output human` (default): Colored text, tables for structured data
- `--output json`: Machine-readable JSON for scripting

## Security

Use a dedicated hot wallet or hardware signer for daily ops. For production agents, issue limited `TradingKeyClaim`s with appropriate `max_usd` and `expiry_days`.
