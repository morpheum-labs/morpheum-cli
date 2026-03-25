use clap::{Args, Subcommand};

use morpheum_sdk_native::inferreg::QuantFormat;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `inferreg` module.
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
    /// Quantization format (fp16, q4km, q5km, q80)
    #[arg(required = true, value_parser = parse_quant_format)]
    pub quant_format: QuantFormat,
}

pub async fn execute(
    cmd: InferenceRegistryQueryCommands,
    dispatcher: Dispatcher,
) -> Result<(), CliError> {
    match cmd {
        InferenceRegistryQueryCommands::Model(args) => {
            query_model(args, &dispatcher).await
        }
        InferenceRegistryQueryCommands::ModelsByQuant(args) => {
            query_models_by_quant(args, &dispatcher).await
        }
        InferenceRegistryQueryCommands::ActiveModels => {
            query_active_models(&dispatcher).await
        }
        InferenceRegistryQueryCommands::Params => {
            query_params(&dispatcher).await
        }
    }
}

async fn query_model(args: ModelArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::inferreg::InferenceRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let model_id = args.model_id.clone();
    let result = client.query_model(args.model_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No model found with ID {model_id}");
    }
    Ok(())
}

async fn query_models_by_quant(
    args: ModelsByQuantArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::inferreg::InferenceRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_models_by_quant(args.quant_format).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active_models(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::inferreg::InferenceRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_active_models().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::inferreg::InferenceRegistryClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    if result.is_some() {
        println!("{json}");
    } else {
        println!("No inference registry parameters configured");
    }
    Ok(())
}

fn parse_quant_format(s: &str) -> Result<QuantFormat, String> {
    match s.to_lowercase().as_str() {
        "fp16" | "f16" => Ok(QuantFormat::Fp16),
        "q4km" | "q4_k_m" => Ok(QuantFormat::Q4KM),
        "q5km" | "q5_k_m" => Ok(QuantFormat::Q5KM),
        "q80" | "q8_0" => Ok(QuantFormat::Q80),
        other => Err(format!(
            "unknown quant format '{other}'; expected: fp16, q4km, q5km, q80"
        )),
    }
}
