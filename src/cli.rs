use clap::{Parser, Subcommand};
use crate::config::OutputFormat;

/// Root CLI structure for the Morpheum command-line interface.
///
/// Single entry point for all commands. Global options are defined here
/// and passed down to every subcommand via the `Dispatcher`.
#[derive(Parser)]
#[command(name = "morpheum")]
#[command(version)]
#[command(about = "Official CLI for Morpheum — the sovereign AI-native L1")]
#[command(long_about = "Full support for mwvm simulation, ERC-8004, MCP, A2A, native x402 payments, GMP bridges, agent lifecycle, and all on-chain registries.")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

/// Global options available to every command.
/// Can also be overridden via environment variables (e.g. `MORPHEUM_CHAIN_ID`).
#[derive(Parser)]
pub struct GlobalArgs {
    /// Chain ID to use (overrides config file)
    #[arg(long, env = "MORPHEUM_CHAIN_ID")]
    pub chain_id: Option<String>,

    /// RPC endpoint URL (overrides config file)
    #[arg(long, env = "MORPHEUM_RPC")]
    pub rpc: Option<String>,

    /// Output format for queries and status commands
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,
}

/// All top-level commands.
///
/// On-chain modules live under `tx` and `query` (14 modules each, 1:1 with Mormcore).
/// Protocol gateways and developer tools are top-level (`mwvm`, `mcp`, `a2a`, `keys`).
///
/// Cross-chain deposit/withdraw lives under `tx bank deposit` / `tx bank withdraw`.
/// Message delivery status: `query gmp delivery`.
/// Full agent registration uses `tx identity register --full`.
#[derive(Subcommand)]
pub enum Commands {
    /// On-chain transaction commands (all 14 modules)
    #[command(subcommand)]
    Tx(crate::tx::TxCommands),

    /// On-chain query commands (mirrors `tx/`)
    #[command(subcommand)]
    Query(crate::query::QueryCommands),

    /// mwvm — Local simulation, debugging, orchestration and developer runtime (Pillar 1)
    #[command(subcommand)]
    Mwvm(crate::mwvm::MwvmCommands),

    /// MCP — Model Context Protocol gateway commands (Pillar 2)
    #[command(subcommand)]
    Mcp(crate::mcp::McpCommands),

    /// A2A — `Agent2Agent` Protocol commands (Pillar 2)
    #[command(subcommand)]
    A2a(crate::a2a::A2aCommands),

    /// Secure key management (native wallets + agent delegation with `TradingKeyClaim`)
    #[command(subcommand)]
    Keys(crate::keys::KeysCommands),

    /// Show current node, chain, and runtime status
    Status,

    /// Configuration management (view, edit, reset)
    #[command(subcommand)]
    Config(crate::config::ConfigCommands),
}
