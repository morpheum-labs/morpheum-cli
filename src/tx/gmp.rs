use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Args, Subcommand};
use morpheum_primitives::address::GOVERNANCE_ADDRESS;

use morpheum_sdk_native::gmp::{
    HyperlaneParamsBuilder, UpdateGmpParamsBuilder, WarpRouteConfigBuilder,
    WarpRouteTransferBuilder,
};
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the GMP (General Message Passing) module.
#[derive(Subcommand)]
pub enum GmpCommands {
    /// Initiate a Warp Route transfer (burn on Morpheum, unlock on destination)
    WarpTransfer(WarpTransferArgs),

    /// Submit GMP governance param update (Hyperlane config, Warp Route config)
    UpdateParams(UpdateParamsArgs),
}

#[derive(Args)]
pub struct WarpTransferArgs {
    /// Hyperlane domain ID of the destination chain
    #[arg(long)]
    pub destination_domain: u32,

    /// 32-byte hex recipient address on the destination chain
    #[arg(long)]
    pub recipient: String,

    /// Bank asset index of the token to transfer
    #[arg(long)]
    pub asset_index: u64,

    /// Amount to transfer
    #[arg(long)]
    pub amount: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct UpdateParamsArgs {
    /// Path to a JSON file defining GMP params (Hyperlane + Warp Route config)
    #[arg(long)]
    pub params_file: PathBuf,

    /// Governance authority address (defaults to deterministic governance module address)
    #[arg(long, default_value = GOVERNANCE_ADDRESS)]
    pub authority: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: GmpCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        GmpCommands::WarpTransfer(args) => warp_transfer(args, &dispatcher).await,
        GmpCommands::UpdateParams(args) => update_params(args, &dispatcher).await,
    }
}

async fn warp_transfer(args: WarpTransferArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = hex::encode(signer.account_id().0);

    let recipient_bytes = hex::decode(&args.recipient)
        .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?;

    let request = WarpRouteTransferBuilder::new()
        .sender(from_address)
        .destination_domain(args.destination_domain)
        .recipient(recipient_bytes)
        .asset_index(args.asset_index)
        .amount(&args.amount)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Warp Route transfer submitted\nDestination domain: {}\nAmount: {} (asset {})\nTxHash: {}",
        args.destination_domain, args.amount, args.asset_index, txhash,
    ));

    Ok(())
}

// ── update-params ───────────────────────────────────────────────────

/// JSON-serializable input for GMP governance params.
#[derive(serde::Deserialize)]
struct ParamsFile {
    hyperlane: Option<HyperlaneParamsInput>,
    warp_route: Option<WarpRouteInput>,
}

#[derive(serde::Deserialize)]
struct HyperlaneParamsInput {
    /// Hex-encoded 20-byte validator addresses.
    validators: Vec<String>,
    threshold: u32,
    #[serde(default)]
    domain_to_caip2: BTreeMap<u32, String>,
    /// Hex-encoded 32-byte trusted sender addresses.
    #[serde(default)]
    trusted_senders: Vec<String>,
}

#[derive(serde::Deserialize)]
struct WarpRouteInput {
    /// Hex-encoded 32-byte recipient address.
    recipient_address: String,
    #[serde(default)]
    routes: BTreeMap<u32, WarpRouteTokenInput>,
}

#[derive(serde::Deserialize)]
struct WarpRouteTokenInput {
    /// Hex-encoded 32-byte collateral address.
    collateral_address: String,
    asset_index: u64,
}

fn decode_hex(label: &str, s: &str) -> Result<Vec<u8>, CliError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|e| CliError::invalid_input(format!("invalid hex for {label}: {e}")))
}

async fn update_params(
    args: UpdateParamsArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let raw = std::fs::read_to_string(&args.params_file).map_err(|e| CliError::Io {
        context: format!("reading params file '{}': {e}", args.params_file.display()),
        source: e,
    })?;
    let input: ParamsFile = serde_json::from_str(&raw)
        .map_err(|e| CliError::invalid_input(format!("invalid params JSON: {e}")))?;

    let mut builder = UpdateGmpParamsBuilder::new().authority(&args.authority);

    if let Some(hl) = input.hyperlane {
        let validators: Vec<Vec<u8>> = hl
            .validators
            .iter()
            .enumerate()
            .map(|(i, v)| decode_hex(&format!("validators[{i}]"), v))
            .collect::<Result<_, _>>()?;

        let trusted_senders: Vec<Vec<u8>> = hl
            .trusted_senders
            .iter()
            .enumerate()
            .map(|(i, s)| decode_hex(&format!("trusted_senders[{i}]"), s))
            .collect::<Result<_, _>>()?;

        let hl_params = HyperlaneParamsBuilder::new()
            .validators(validators)
            .threshold(hl.threshold)
            .domain_to_caip2(hl.domain_to_caip2)
            .trusted_senders(trusted_senders)
            .build()
            .map_err(CliError::Sdk)?;

        builder = builder.hyperlane(hl_params);
    }

    if let Some(wr) = input.warp_route {
        let recipient = decode_hex("warp_route.recipient_address", &wr.recipient_address)?;

        let mut wr_builder = WarpRouteConfigBuilder::new().recipient_address(recipient);

        for (domain, token) in &wr.routes {
            let collateral =
                decode_hex(&format!("routes[{domain}].collateral_address"), &token.collateral_address)?;
            wr_builder = wr_builder.add_route(*domain, collateral, token.asset_index);
        }

        let warp_config = wr_builder.build().map_err(CliError::Sdk)?;
        builder = builder.warp_route(warp_config);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "GMP params updated\nAuthority: {}\nTxHash: {}",
        args.authority, txhash,
    ));

    Ok(())
}
