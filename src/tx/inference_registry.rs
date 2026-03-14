use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::inference_registry::{RegisterModelBuilder, QuantFormat};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `inference_registry` module.
///
/// Manages on-chain model commitments with zk verification support —
/// the AI primitive layer that backs the mwvm runtime (Pillar 1).
#[derive(Subcommand)]
pub enum InferenceRegistryCommands {
    /// Register a new model commitment on-chain
    RegisterModel(RegisterModelArgs),
}

#[derive(Args)]
pub struct RegisterModelArgs {
    /// Display name for the model (e.g. "ZK-LLAMA-8B-Q4")
    #[arg(long)]
    pub name: String,

    /// Quantization format (q4km, q5km, q80, fp16)
    #[arg(long, value_parser = parse_quant_format)]
    pub quant: QuantFormat,

    /// Parameter count (e.g. `8_000_000_000` for 8B)
    #[arg(long)]
    pub param_count: u64,

    /// ZK commitment hash as hex
    #[arg(long)]
    pub zk_commitment: String,

    /// Supported operations bitflags (1=infer, 2=embed, 4=vector-search, 8=fine-tune)
    #[arg(long, default_value = "1")]
    pub supported_ops: u64,

    /// Model version number
    #[arg(long, default_value = "1")]
    pub version: u32,

    /// Optional weights payload as hex
    #[arg(long)]
    pub weights_payload: Option<String>,

    /// Key name to sign with (authority key)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(
    cmd: InferenceRegistryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        InferenceRegistryCommands::RegisterModel(args) => {
            register_model(args, &dispatcher).await
        }
    }
}

async fn register_model(
    args: RegisterModelArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let authority = hex::encode(signer.account_id().0);

    let zk_commitment = hex::decode(&args.zk_commitment)
        .map_err(|e| CliError::invalid_input(format!("invalid hex for zk_commitment: {e}")))?;

    let weights_payload = args
        .weights_payload
        .as_deref()
        .map(hex::decode)
        .transpose()
        .map_err(|e| CliError::invalid_input(format!("invalid hex for weights_payload: {e}")))?
        .unwrap_or_default();

    let request = RegisterModelBuilder::new()
        .authority(&authority)
        .display_name(&args.name)
        .quant_format(args.quant)
        .param_count(args.param_count)
        .zk_commitment(zk_commitment)
        .supported_ops(args.supported_ops)
        .version(args.version)
        .weights_payload(weights_payload)
        .build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    ).await?;

    dispatcher.output.success(format!(
        "Model registered: {} ({:?}, {}B params)\nTxHash: {}",
        args.name, args.quant, args.param_count, txhash,
    ));

    Ok(())
}

fn parse_quant_format(s: &str) -> Result<QuantFormat, String> {
    match s.to_lowercase().as_str() {
        "q4km" | "q4_k_m" => Ok(QuantFormat::Q4KM),
        "q5km" | "q5_k_m" => Ok(QuantFormat::Q5KM),
        "q80" | "q8_0" => Ok(QuantFormat::Q80),
        "fp16" | "f16" => Ok(QuantFormat::Fp16),
        other => Err(format!(
            "unknown quant format '{other}'; expected: q4km, q5km, q80, fp16"
        )),
    }
}
