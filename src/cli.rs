use crate::config::OutputFormat;
use clap::{Parser, Subcommand};
#[cfg(feature = "_tx")]
use morpheum_signing_native::{TxGasLimit, DEFAULT_GAS_LIMIT, TX_GAS_BUDGET};

/// Root CLI structure for the Morpheum command-line interface.
///
/// Single entry point for all commands. Global options are defined here
/// and passed down to every subcommand via the `Dispatcher`.
#[derive(Parser)]
#[command(name = "morpheum")]
#[command(version)]
#[command(about = "Official CLI for Morpheum — the sovereign AI-native L1")]
#[command(
    long_about = "Full support for mwvm simulation, ERC-8004, MCP, A2A, native x402 payments, GMP bridges, agent lifecycle, and all on-chain registries."
)]
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

    /// Gas limit declared by every transaction this command signs (range and
    /// default: see --help)
    #[cfg(feature = "_tx")]
    #[arg(
        long,
        global = true,
        value_parser = parse_gas_limit,
        long_help = gas_limit_long_help()
    )]
    pub gas_limit: Option<TxGasLimit>,
}

/// The `--gas-limit` long help, built from the constants it names, so the
/// range and the default it prints are the ones the CLI applies.
#[cfg(feature = "_tx")]
fn gas_limit_long_help() -> String {
    format!(
        "Gas limit declared by every transaction this command signs, in \
         1..={TX_GAS_BUDGET}\n\n\
         The limit is signed. Execution fails once a transaction uses more, and the \
         whole limit is reserved in its block whether it is used or not, so declare \
         what the transaction needs.\n\n\
         When unset, a transaction declares {default} (the signing SDK's \
         DEFAULT_GAS_LIMIT), which suits native-module messages, or the gas limit \
         its message states for itself when that is larger (an SVM program call \
         states its compute limit). Contract deployments and calls, including \
         `tx bank withdraw`, can need more than either. A value below the \
         message's own limit is refused.",
        default = DEFAULT_GAS_LIMIT.get(),
    )
}

/// Parses `--gas-limit` through [`TxGasLimit::new`], the validity rule the
/// chain applies, so a value the chain would refuse (`0`, or above
/// [`TX_GAS_BUDGET`]) is refused here, before a key is loaded or a node is
/// contacted. Every refusal names the valid range.
#[cfg(feature = "_tx")]
fn parse_gas_limit(raw: &str) -> Result<TxGasLimit, String> {
    let declared: u64 = raw
        .parse()
        .map_err(|err| format!("{err} (a gas limit is a whole number in 1..={TX_GAS_BUDGET})"))?;
    TxGasLimit::new(declared).map_err(|refusal| refusal.to_string())
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

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    /// Every command and argument in the tree passes clap's own consistency
    /// checks, so a global flag (`--gas-limit` included) collides with no
    /// subcommand's argument. Parsing checks only the commands it reaches;
    /// this checks all of them.
    #[test]
    fn command_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[cfg(feature = "_tx")]
    fn gas_limit(argv: &[&str]) -> Result<Option<TxGasLimit>, clap::Error> {
        Cli::try_parse_from(argv).map(|cli| cli.global.gas_limit)
    }

    /// A gas limit the chain would refuse is refused while the command line
    /// is parsed, so no transaction is signed with it, and the refusal tells
    /// the user which values are valid.
    #[cfg(feature = "_tx")]
    #[test]
    fn gas_limit_flag_rejects_zero_and_over_budget() {
        use clap::error::ErrorKind;

        let range = format!("1..={TX_GAS_BUDGET}");
        let over_budget = (TX_GAS_BUDGET + 1).to_string();
        for value in ["0", over_budget.as_str(), "lots"] {
            let err = gas_limit(&["morpheum", "status", "--gas-limit", value])
                .expect_err("an invalid gas limit must not parse");
            assert_eq!(err.kind(), ErrorKind::ValueValidation, "{value}: {err}");
            assert!(
                err.to_string().contains(&range),
                "{value}: the refusal must name the valid range: {err}",
            );
        }
    }

    /// Both ends of the range parse, so the CLI adds no bound of its own, and
    /// the flag is accepted before or after the subcommand, so it can follow
    /// the arguments of the transaction it applies to.
    #[cfg(feature = "_tx")]
    #[test]
    fn gas_limit_flag_accepts_the_whole_range_anywhere_in_the_command() {
        let budget = TX_GAS_BUDGET.to_string();
        assert_eq!(gas_limit(&["morpheum", "status"]).expect("parses"), None);
        assert_eq!(
            gas_limit(&["morpheum", "--gas-limit", "1", "status"]).expect("parses"),
            Some(TxGasLimit::MIN),
        );
        assert_eq!(
            gas_limit(&["morpheum", "config", "show", "--gas-limit", &budget]).expect("parses"),
            Some(TxGasLimit::MAX),
        );
    }

    /// `--help` names the valid range and the default, both taken from the
    /// constants the CLI applies.
    #[cfg(feature = "_tx")]
    #[test]
    fn gas_limit_help_names_the_range_and_the_default() {
        let command = Cli::command();
        let help = command
            .get_arguments()
            .find(|arg| arg.get_id() == "gas_limit")
            .and_then(clap::Arg::get_long_help)
            .expect("--gas-limit has a long help")
            .to_string();
        for named in [
            format!("1..={TX_GAS_BUDGET}"),
            DEFAULT_GAS_LIMIT.get().to_string(),
        ] {
            assert!(help.contains(&named), "the help must name {named}: {help}");
        }
    }
}
