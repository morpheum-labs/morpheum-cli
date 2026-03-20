use std::path::PathBuf;

use clap::{Args, Subcommand};

use morpheum_sdk_cosmwasm::{
    ExecuteContractBuilder, InstantiateContractBuilder, StoreCodeBuilder,
};
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for CosmWasm contracts on Morpheum's embedded VM.
#[derive(Subcommand)]
pub enum CosmwasmCommands {
    /// Upload WASM bytecode to the chain
    StoreCode(StoreCodeArgs),

    /// Instantiate a contract from stored code
    Instantiate(InstantiateArgs),

    /// Execute a message on a deployed contract
    Execute(ExecuteArgs),
}

#[derive(Args)]
pub struct StoreCodeArgs {
    /// Path to the .wasm file
    #[arg(long)]
    pub wasm_file: PathBuf,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct InstantiateArgs {
    /// Code ID of the stored WASM module
    #[arg(long)]
    pub code_id: u64,

    /// Human-readable label for the contract
    #[arg(long)]
    pub label: String,

    /// JSON-encoded instantiation message
    #[arg(long)]
    pub msg: String,

    /// Admin address (if omitted, contract has no admin)
    #[arg(long)]
    pub admin: Option<String>,

    /// Funds to send with instantiation (format: "amount:denom", repeatable)
    #[arg(long)]
    pub funds: Vec<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct ExecuteArgs {
    /// Contract address (morm1...)
    #[arg(long)]
    pub contract: String,

    /// JSON-encoded execute message
    #[arg(long)]
    pub msg: String,

    /// Funds to send with execution (format: "amount:denom", repeatable)
    #[arg(long)]
    pub funds: Vec<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: CosmwasmCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        CosmwasmCommands::StoreCode(args) => store_code(args, &dispatcher).await,
        CosmwasmCommands::Instantiate(args) => instantiate(args, &dispatcher).await,
        CosmwasmCommands::Execute(args) => execute_contract(args, &dispatcher).await,
    }
}

async fn store_code(args: StoreCodeArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let sender = hex::encode(signer.account_id().0);

    let wasm_byte_code = std::fs::read(&args.wasm_file).map_err(|e| CliError::Io {
        context: format!("reading WASM file '{}': {e}", args.wasm_file.display()),
        source: e,
    })?;

    let size_kb = wasm_byte_code.len() / 1024;

    let request = StoreCodeBuilder::new()
        .sender(&sender)
        .wasm_byte_code(wasm_byte_code)
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
        "Code stored\nWASM file: {}\nSize: {size_kb} KB\nTxHash: {txhash}",
        args.wasm_file.display(),
    ));

    Ok(())
}

async fn instantiate(args: InstantiateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let sender = hex::encode(signer.account_id().0);

    let msg_bytes: Vec<u8> = args.msg.as_bytes().to_vec();
    serde_json::from_slice::<serde_json::Value>(&msg_bytes)
        .map_err(|e| CliError::invalid_input(format!("--msg is not valid JSON: {e}")))?;

    let mut builder = InstantiateContractBuilder::new()
        .sender(&sender)
        .code_id(args.code_id)
        .label(&args.label)
        .msg(msg_bytes);

    if let Some(ref admin) = args.admin {
        builder = builder.admin(admin);
    }

    for fund_str in &args.funds {
        let (denom, amount) = parse_fund(fund_str)?;
        builder = builder.add_funds(denom, amount);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Contract instantiated\nCode ID: {}\nLabel: {}\nTxHash: {txhash}",
        args.code_id, args.label,
    ));

    Ok(())
}

async fn execute_contract(args: ExecuteArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let sender = hex::encode(signer.account_id().0);

    let msg_bytes: Vec<u8> = args.msg.as_bytes().to_vec();
    serde_json::from_slice::<serde_json::Value>(&msg_bytes)
        .map_err(|e| CliError::invalid_input(format!("--msg is not valid JSON: {e}")))?;

    let mut builder = ExecuteContractBuilder::new()
        .sender(&sender)
        .contract(&args.contract)
        .msg(msg_bytes);

    for fund_str in &args.funds {
        let (denom, amount) = parse_fund(fund_str)?;
        builder = builder.add_funds(denom, amount);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Contract executed\nContract: {}\nTxHash: {txhash}",
        args.contract,
    ));

    Ok(())
}

/// Parses "amount:denom" into (denom, amount) strings.
fn parse_fund(s: &str) -> Result<(&str, &str), CliError> {
    let (amount, denom) = s
        .split_once(':')
        .ok_or_else(|| CliError::invalid_input(
            format!("invalid --funds format '{s}': expected 'amount:denom' (e.g. '1000000:umorm')"),
        ))?;
    if amount.is_empty() || denom.is_empty() {
        return Err(CliError::invalid_input(
            format!("invalid --funds format '{s}': both amount and denom must be non-empty"),
        ));
    }
    Ok((denom, amount))
}
