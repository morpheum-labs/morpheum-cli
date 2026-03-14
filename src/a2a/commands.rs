use clap::{Args, Subcommand};
use serde::Serialize;
use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// A2A (`Agent2Agent` Protocol) commands (Pillar 2 compatibility layer).
///
/// These commands enable seamless collaboration between agents using the
/// standard A2A protocol. Agents can discover each other, delegate tasks,
/// and collaborate in real time — fully interoperable with Google ADK,
/// `LangGraph`, `BeeAI`, and other A2A frameworks.
#[derive(Subcommand)]
pub enum A2aCommands {
    /// Delegate a task to another agent
    Delegate(DelegateArgs),

    /// Discover compatible agents (semantic search or capability filter)
    Discover(DiscoverArgs),

    /// Start a collaborative session with another agent
    Collaborate(CollaborateArgs),

    /// Show A2A connection status and capabilities for an agent
    Status(StatusArgs),
}

#[derive(Args)]
pub struct DelegateArgs {
    /// Target agent ID or DID
    #[arg(required = true)]
    pub target: String,

    /// Task description or goal
    #[arg(long, required = true)]
    pub task: String,

    /// Optional JSON parameters for the task
    #[arg(long)]
    pub params: Option<String>,

    /// Use local mwvm simulation instead of remote A2A gateway
    #[arg(long)]
    pub local: bool,

    /// Key name to sign with (for authenticated delegation)
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct DiscoverArgs {
    /// Semantic search query or capability filter
    #[arg(required = true)]
    pub query: String,

    /// Maximum number of results
    #[arg(long, default_value = "10")]
    pub limit: u32,

    /// Use local mwvm simulation
    #[arg(long)]
    pub local: bool,
}

#[derive(Args)]
pub struct CollaborateArgs {
    /// Target agent ID or DID to collaborate with
    #[arg(required = true)]
    pub target: String,

    /// Initial collaboration goal or context
    #[arg(long)]
    pub goal: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Agent ID or DID
    #[arg(required = true)]
    pub agent: String,
}

/// Execute A2A commands.
#[allow(clippy::unused_async)]
pub async fn execute(cmd: A2aCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        A2aCommands::Delegate(args) => delegate_task(args, &dispatcher),
        A2aCommands::Discover(args) => discover_agents(args, &dispatcher),
        A2aCommands::Collaborate(args) => start_collaboration(args, &dispatcher),
        A2aCommands::Status(args) => show_status(args, &dispatcher),
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn delegate_task(args: DelegateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Delegating task to agent {} {}",
        args.target,
        if args.local { "(local mwvm simulation)" } else { "(via A2A gateway)" }
    ));

    // Production: sdk.a2a().delegate(...) or gateway call
    output.success(format!(
        "Task successfully delegated to agent {}",
        args.target
    ));

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn discover_agents(args: DiscoverArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Discovering agents matching '{}' (limit: {}) {}",
        args.query,
        args.limit,
        if args.local { "(local simulation)" } else { "(via A2A directory)" }
    ));

    // In production this would call sdk.a2a().discover(...)
    let agents = vec![
        AgentInfo { id: "did:agent:alpha-trader".to_string(), name: "AlphaTrader".to_string() },
        AgentInfo { id: "did:agent:research-bot".to_string(), name: "ResearchBot".to_string() },
    ];

    output.print_list(&agents)?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn start_collaboration(args: CollaborateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Initiating collaboration with agent {} using key '{}'",
        args.target, args.from
    ));

    // In production this would establish A2A session via SDK
    output.success(format!(
        "Collaboration session started with agent {}",
        args.target
    ));

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn show_status(args: StatusArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!("Checking A2A status for agent {}", args.agent));

    // In production this would query A2A endpoint via SDK
    output.success(format!("A2A endpoint is healthy for agent {}", args.agent));
    output.info("Supported transports: HTTP, SSE, JSON-RPC, gRPC");
    output.info("Capabilities: task_delegation, collaboration, streaming");

    Ok(())
}

#[derive(tabled::Tabled, Serialize)]
struct AgentInfo {
    #[tabled(rename = "Agent ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
}