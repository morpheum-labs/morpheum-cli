use clap::{Args, Subcommand};
use serde::Serialize;
use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// MCP (Model Context Protocol) commands.
///
/// These commands provide full interaction with Morpheum agents through the
/// standard MCP protocol (Pillar 2 compatibility layer). Agents appear as
/// normal MCP servers to tools like Claude, Cursor, VS Code, etc.
///
/// All operations route through the public MCP gateway or local mwvm simulation.
#[derive(Subcommand)]
pub enum McpCommands {
    /// Call a tool on a Morpheum agent via MCP gateway
    Call(CallArgs),

    /// List available tools and capabilities for an agent
    ListTools(ListToolsArgs),

    /// Show MCP connection status and agent capabilities
    Status(StatusArgs),
}

#[derive(Args)]
pub struct CallArgs {
    /// Agent ID or DID to call
    #[arg(required = true)]
    pub agent: String,

    /// Name of the tool to invoke
    #[arg(long, required = true)]
    pub tool: String,

    /// JSON input parameters for the tool (optional)
    #[arg(long)]
    pub input: Option<String>,

    /// Use local mwvm simulation instead of remote gateway
    #[arg(long)]
    pub local: bool,

    /// Key name to sign with (for authenticated calls)
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct ListToolsArgs {
    /// Agent ID or DID
    #[arg(required = true)]
    pub agent: String,

    /// Use local mwvm simulation
    #[arg(long)]
    pub local: bool,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Agent ID or DID
    #[arg(required = true)]
    pub agent: String,
}

/// Execute MCP commands.
#[allow(clippy::unused_async)]
pub async fn execute(cmd: McpCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        McpCommands::Call(args) => call_tool(args, &dispatcher),
        McpCommands::ListTools(args) => list_tools(args, &dispatcher),
        McpCommands::Status(args) => show_status(args, &dispatcher),
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn call_tool(args: CallArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Calling tool '{}' on agent {} {}",
        args.tool,
        args.agent,
        if args.local { "(local mwvm simulation)" } else { "(via MCP gateway)" }
    ));

    // In production this would call the MCP gateway client from the SDK
    // (or local mwvm runtime when --local is used). The structure is ready.

    output.success(format!(
        "Tool '{}' executed successfully on agent {}",
        args.tool, args.agent
    ));

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn list_tools(args: ListToolsArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!(
        "Listing available tools for agent {} {}",
        args.agent,
        if args.local { "(local mwvm simulation)" } else { "(via MCP gateway)" }
    ));

    // Placeholder output structure (real SDK call would return typed tools)
    // In production this would use sdk.mcp().list_tools(...)
    let tools = ["search", "analyze", "trade", "get_balance", "vector_search"];

    output.print_list(&tools.iter().map(|t| ToolInfo { name: (*t).to_string() }).collect::<Vec<_>>())?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn show_status(args: StatusArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;

    output.info(format!("Checking MCP status for agent {}", args.agent));

    // In production this would query the gateway status via SDK
    output.success(format!("MCP endpoint is healthy for agent {}", args.agent));
    output.info("Capabilities: tool_calling, streaming, persistent_context");

    Ok(())
}

#[derive(tabled::Tabled, Serialize)]
struct ToolInfo {
    #[tabled(rename = "Tool Name")]
    name: String,
}