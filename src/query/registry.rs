use clap::{Args, Subcommand};
use morpheum_sdk_core::ChainRegistryOps as _;
use morpheum_sdk_evm::config::{ChainRegistry, TokenType};
use morpheum_sdk_svm::config::{SolanaChainRegistry, SvmTokenType};
use serde::Serialize;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::xchain::ChainSpec;

/// Query the SDK chain/token registries.
///
/// Data is sourced directly from the same `chains.toml` files that
/// `tx bank deposit/withdraw` uses. No manual maintenance needed --
/// adding a chain or token in the SDK automatically surfaces it here.
#[derive(Subcommand)]
pub enum RegistryQueryCommands {
    /// List all supported external chains (EVM + SVM)
    Chains,

    /// List tokens available on a specific chain
    Tokens(TokensArgs),

    /// List all chains that support a given token
    Routes(RoutesArgs),
}

#[derive(Args)]
pub struct TokensArgs {
    /// Chain spec in `<vm>:<network>` format (e.g. `evm:sepolia`, `svm:devnet`)
    #[arg(long)]
    pub chain: String,
}

#[derive(Args)]
pub struct RoutesArgs {
    /// Token symbol (e.g. USDC, ETH, SOL)
    #[arg(long)]
    pub token: String,
}

// ── JSON output types ──────────────────────────────────────────────

#[derive(Serialize)]
struct ChainEntry {
    name: String,
    vm: &'static str,
    spec: String,
    hyperlane_domain: u32,
    tokens: Vec<String>,
}

#[derive(Serialize)]
struct TokenEntry {
    symbol: String,
    decimals: u8,
    morpheum_asset_index: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_type: Option<String>,
    actions: Vec<&'static str>,
}

#[derive(Serialize)]
struct RouteEntry {
    spec: String,
    vm: &'static str,
    decimals: u8,
    morpheum_asset_index: u64,
    actions: Vec<&'static str>,
}

// ── Execution ──────────────────────────────────────────────────────

pub fn execute(cmd: RegistryQueryCommands, dispatcher: &Dispatcher) -> Result<(), CliError> {
    match cmd {
        RegistryQueryCommands::Chains => chains(dispatcher),
        RegistryQueryCommands::Tokens(args) => tokens(&args, dispatcher),
        RegistryQueryCommands::Routes(args) => routes(&args, dispatcher),
    }
}

fn load_evm_registry() -> Result<ChainRegistry, CliError> {
    ChainRegistry::load_with_defaults(morpheum_sdk_evm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("EVM", format!("load chain registry: {e}")))
}

fn load_svm_registry() -> Result<SolanaChainRegistry, CliError> {
    SolanaChainRegistry::load_with_defaults(morpheum_sdk_svm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("SVM", format!("load chain registry: {e}")))
}

fn derive_evm_actions(warp_route: Option<&String>) -> Vec<&'static str> {
    if warp_route.is_some() {
        vec!["bank.deposit", "bank.withdraw"]
    } else {
        vec![]
    }
}

fn derive_svm_actions(warp_route: Option<&String>) -> Vec<&'static str> {
    if warp_route.is_some() {
        vec!["bank.deposit", "bank.withdraw"]
    } else {
        vec![]
    }
}

fn evm_token_type_label(t: &TokenType) -> Option<String> {
    match t {
        TokenType::Native => Some("native".to_string()),
        TokenType::Erc20 => None,
    }
}

fn svm_token_type_label(t: &SvmTokenType) -> Option<String> {
    match t {
        SvmTokenType::Native => Some("native".to_string()),
        SvmTokenType::Spl => None,
    }
}

fn chains(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let evm = load_evm_registry()?;
    let svm = load_svm_registry()?;

    let mut entries: Vec<ChainEntry> = Vec::new();

    let mut evm_names: Vec<_> = evm.chains.keys().collect();
    evm_names.sort();
    for name in evm_names {
        let chain = &evm.chains[name];
        let mut token_names: Vec<_> = chain.tokens.keys().cloned().collect();
        token_names.sort();
        entries.push(ChainEntry {
            name: name.clone(),
            vm: "evm",
            spec: format!("evm:{name}"),
            hyperlane_domain: chain.hyperlane_domain,
            tokens: token_names,
        });
    }

    let mut svm_names: Vec<_> = svm.chains.keys().collect();
    svm_names.sort();
    for name in svm_names {
        let chain = &svm.chains[name];
        let mut token_names: Vec<_> = chain.tokens.keys().cloned().collect();
        token_names.sort();
        entries.push(ChainEntry {
            name: name.clone(),
            vm: "svm",
            spec: format!("svm:{name}"),
            hyperlane_domain: chain.hyperlane_domain,
            tokens: token_names,
        });
    }

    dispatcher
        .output
        .print_json(&entries)
        .map_err(|e| CliError::invalid_input(format!("JSON output: {e}")))?;
    Ok(())
}

fn tokens(args: &TokensArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let spec = ChainSpec::parse(&args.chain)?;
    let mut entries: Vec<TokenEntry> = Vec::new();

    match spec.chain_type {
        crate::xchain::ChainType::Evm => {
            let registry = load_evm_registry()?;
            let chain = registry.get_chain(&spec.network).ok_or_else(|| {
                CliError::chain("EVM", format!("unknown chain: '{}'", spec.network))
            })?;
            let mut names: Vec<_> = chain.tokens.keys().collect();
            names.sort();
            for symbol in names {
                let tok = &chain.tokens[symbol];
                entries.push(TokenEntry {
                    symbol: symbol.clone(),
                    decimals: tok.decimals,
                    morpheum_asset_index: tok.morpheum_asset_index,
                    token_type: evm_token_type_label(&tok.token_type),
                    actions: derive_evm_actions(tok.morpheum_warp_route.as_ref()),
                });
            }
        }
        crate::xchain::ChainType::Svm => {
            let registry = load_svm_registry()?;
            let chain = registry.get_chain(&spec.network).ok_or_else(|| {
                CliError::chain("SVM", format!("unknown chain: '{}'", spec.network))
            })?;
            let mut names: Vec<_> = chain.tokens.keys().collect();
            names.sort();
            for symbol in names {
                let tok = &chain.tokens[symbol];
                entries.push(TokenEntry {
                    symbol: symbol.clone(),
                    decimals: tok.decimals,
                    morpheum_asset_index: tok.morpheum_asset_index,
                    token_type: svm_token_type_label(&tok.token_type),
                    actions: derive_svm_actions(tok.morpheum_warp_route.as_ref()),
                });
            }
        }
    }

    dispatcher
        .output
        .print_json(&entries)
        .map_err(|e| CliError::invalid_input(format!("JSON output: {e}")))?;
    Ok(())
}

fn routes(args: &RoutesArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let upper = args.token.to_ascii_uppercase();
    let evm = load_evm_registry()?;
    let svm = load_svm_registry()?;

    let mut entries: Vec<RouteEntry> = Vec::new();

    let mut evm_names: Vec<_> = evm.chains.keys().collect();
    evm_names.sort();
    for name in evm_names {
        let chain = &evm.chains[name];
        if let Some(tok) = chain.tokens.get(&upper) {
            entries.push(RouteEntry {
                spec: format!("evm:{name}"),
                vm: "evm",
                decimals: tok.decimals,
                morpheum_asset_index: tok.morpheum_asset_index,
                actions: derive_evm_actions(tok.morpheum_warp_route.as_ref()),
            });
        }
    }

    let mut svm_names: Vec<_> = svm.chains.keys().collect();
    svm_names.sort();
    for name in svm_names {
        let chain = &svm.chains[name];
        if let Some(tok) = chain.tokens.get(&upper) {
            entries.push(RouteEntry {
                spec: format!("svm:{name}"),
                vm: "svm",
                decimals: tok.decimals,
                morpheum_asset_index: tok.morpheum_asset_index,
                actions: derive_svm_actions(tok.morpheum_warp_route.as_ref()),
            });
        }
    }

    if entries.is_empty() {
        return Err(CliError::invalid_input(format!(
            "no routes found for token '{}'",
            args.token
        )));
    }

    dispatcher
        .output
        .print_json(&entries)
        .map_err(|e| CliError::invalid_input(format!("JSON output: {e}")))?;
    Ok(())
}
