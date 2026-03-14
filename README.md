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
# Setup wizard (creates ~/.morpheum, validates mnemonic but never stores it)
morpheum setup

# Issue delegated VC (dry-run by default; no mnemonic needed)
morpheum vc issue --agent 0xabc... --permissions TRADE --max-usd 10000 --expiry-days 90

# Actually broadcast (requires MORPHEUM_MNEMONIC, prompts for confirmation)
morpheum vc issue --agent 0xabc... --broadcast

# Non-interactive (scripts): skip confirmation
morpheum vc issue --agent 0xabc... --broadcast --yes

# JSON output (for scripting)
morpheum --output json vc issue --agent 0xabc... --dry-run

# Create a market order (table output)
morpheum market order --market-id 42 --side buy --size 5 --price 42069
```

## Config

- Default config path: `~/.morpheum/config.toml`
- Override with `--config` or `MORPHEUM_CONFIG` env
- **Mnemonic**: `MORPHEUM_MNEMONIC` env only — never stored in config (security)

## Output formats

- `--output human` (default): Colored text, tables for structured data
- `--output json`: Machine-readable JSON for scripting

## Security

Use a dedicated hot wallet or hardware signer for daily ops. For production agents, issue limited `TradingKeyClaim`s with appropriate `max_usd` and `expiry_days`.
