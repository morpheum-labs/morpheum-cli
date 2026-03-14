use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;
use morpheum_sdk_native::inference_registry::types::QuantFormat;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

/// Query commands for the `inference_registry` module.
///
/// Read-only access to on-chain model commitments, quantization format views,
/// active model listings, and module parameters.
#[derive(Subcommand)]
pub enum InferenceRegistryQueryCommands {
    /// Get a specific model commitment by model ID
    Model(ModelArgs),

    /// List models filtered by quantization format
    ModelsByQuant(ModelsByQuantArgs),

    /// List all currently active models
    ActiveModels,

    /// Get the current inference registry module parameters
    Params,
}

#[derive(Args)]
pub struct ModelArgs {
    /// Model ID (hex-encoded)
    #[arg(required = true)]
    pub model_id: String,
}

#[derive(Args)]
pub struct ModelsByQuantArgs {
    /// Quantization format (fp16, q4km, q5km, q8, gguf)
    #[arg(required = true, value_parser = parse_quant_format)]
    pub quant_format: QuantFormat,
}

pub async fn execute(
    cmd: InferenceRegistryQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        InferenceRegistryQueryCommands::Model(args) => {
            query_model(args, &sdk, &dispatcher.output).await
        }
        InferenceRegistryQueryCommands::ModelsByQuant(args) => {
            query_models_by_quant(args, &sdk, &dispatcher.output).await
        }
        InferenceRegistryQueryCommands::ActiveModels => {
            query_active_models(&sdk, &dispatcher.output).await
        }
        InferenceRegistryQueryCommands::Params => {
            query_params(&sdk, &dispatcher.output).await
        }
    }
}

async fn query_model(
    args: ModelArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.inference_registry()
        .query_and_print_optional(
            output,
            &format!("No model found with ID {}", args.model_id),
            |c| async move { c.query_model(&args.model_id).await },
        )
        .await
}

async fn query_models_by_quant(
    args: ModelsByQuantArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.inference_registry()
        .query_and_print_list(output, |c| async move {
            c.query_models_by_quant(args.quant_format).await
        })
        .await
}

async fn query_active_models(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.inference_registry()
        .query_and_print_list(output, |c| async move { c.query_active_models().await })
        .await
}

async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.inference_registry()
        .query_and_print_optional(
            output,
            "No inference registry parameters configured",
            |c| async move { c.query_params().await },
        )
        .await
}

fn parse_quant_format(s: &str) -> Result<QuantFormat, String> {
    match s.to_lowercase().as_str() {
        "fp16" => Ok(QuantFormat::Fp16),
        "q4km" | "q4_k_m" => Ok(QuantFormat::Q4KM),
        "q5km" | "q5_k_m" => Ok(QuantFormat::Q5KM),
        "q8" | "q8_0" => Ok(QuantFormat::Q8),
        "gguf" => Ok(QuantFormat::Gguf),
        other => Err(format!(
            "unknown quant format '{other}'; expected: fp16, q4km, q5km, q8, gguf"
        )),
    }
}
