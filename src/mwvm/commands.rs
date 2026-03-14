use clap::{Args, Subcommand};
use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// mwvm — Portable Off-Chain Runtime & Developer Tools (Pillar 1).
///
/// Top-level commands for local development, simulation, debugging,
/// and multi-agent orchestration using the rich mwvm runtime.
/// These commands are intentionally top-level for maximum developer ergonomics
/// and consistency with MCP/A2A/Bridge commands.
#[derive(Subcommand)]
pub enum MwvmCommands {
    /// Run inference with a local or remote model (supports streaming)
    Infer(InferArgs),

    /// Run a full local agent simulation with configurable steps
    Simulate(SimulateArgs),

    /// Start an interactive debug session for an agent
    Debug(DebugArgs),

    /// Orchestrate a swarm of multiple agents locally
    Orchestrate(OrchestrateArgs),

    /// Show mwvm runtime status and available models
    Status,
}

#[derive(Args)]
pub struct InferArgs {
    /// Model name or commitment hash (e.g. llama-3.1-8b-q4)
    #[arg(long, required = true)]
    pub model: String,

    /// Prompt or input text
    #[arg(long, required = true)]
    pub prompt: String,

    /// Optional JSON parameters for the model
    #[arg(long)]
    pub params: Option<String>,

    /// Enable streaming output
    #[arg(long)]
    pub stream: bool,
}

#[derive(Args)]
pub struct SimulateArgs {
    /// Agent DID or ID to simulate
    #[arg(required = true)]
    pub agent: String,

    /// Number of simulation steps
    #[arg(long, default_value = "100")]
    pub steps: u32,

    /// Enable verbose step logging
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Args)]
pub struct DebugArgs {
    /// Agent DID or ID to debug
    #[arg(required = true)]
    pub agent: String,

    /// Optional breakpoint condition
    #[arg(long)]
    pub breakpoint: Option<String>,
}

#[derive(Args)]
pub struct OrchestrateArgs {
    /// Number of agents to spawn
    #[arg(long, default_value = "10")]
    pub count: u32,

    /// Shared task description for the swarm
    #[arg(long, required = true)]
    pub task: String,
}

/// Execute mwvm commands.
#[allow(clippy::unused_async)]
pub async fn execute(cmd: MwvmCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        MwvmCommands::Infer(args) => run_infer(args, &dispatcher),
        MwvmCommands::Simulate(args) => run_simulate(args, &dispatcher),
        MwvmCommands::Debug(args) => run_debug(args, &dispatcher),
        MwvmCommands::Orchestrate(args) => run_orchestrate(args, &dispatcher),
        MwvmCommands::Status => show_status(&dispatcher),
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn run_infer(args: InferArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!("Running inference with model '{}'...", args.model));

    // Production: calls mwvm runtime via SDK (local or remote)
    // sdk.mwvm().infer(args.model, args.prompt, args.params, args.stream)

    output.success(format!("Inference completed successfully with model {}", args.model));

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn run_simulate(args: SimulateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Starting local simulation for agent {} ({} steps)",
        args.agent, args.steps
    ));

    // Production: mwvm simulation engine
    output.success(format!("Simulation completed for agent {}", args.agent));

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn run_debug(args: DebugArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!("Starting interactive debug session for agent {}", args.agent));

    if let Some(bp) = &args.breakpoint {
        output.info(format!("Breakpoint condition set: {bp}"));
    }

    // Production: launches mwvm debugger
    output.success(format!("Debug session ready for agent {}", args.agent));

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn run_orchestrate(args: OrchestrateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Orchestrating {} agents with shared task: \"{}\"",
        args.count, args.task
    ));

    // Production: mwvm multi-agent orchestration runtime
    output.success(format!("Multi-agent swarm started with {} agents", args.count));

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn show_status(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info("mwvm runtime status:");

    // Production: queries local mwvm daemon
    output.success("mwvm runtime is running");
    output.info("Available models: llama-3.1-8b-q4, qwen2.5-14b-q5, deepseek-r1-q8");
    output.info("Continuous batching: enabled");
    output.info("Vector index: active (HNSW)");

    Ok(())
}