use clap::{Args, Subcommand};

use morpheum_sdk_native::job::JobState;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Query commands for the `job` module (ERC-8183 compatible).
///
/// Read-only access to jobs, role-based views (client/provider/evaluator),
/// state filtering, active jobs, and module parameters.
#[derive(Subcommand)]
pub enum JobQueryCommands {
    /// Get a specific job by ID
    Get(GetArgs),

    /// List jobs by client agent (paginated)
    ByClient(ByRoleArgs),

    /// List jobs by provider agent (paginated)
    ByProvider(ByRoleArgs),

    /// List jobs by evaluator agent (paginated)
    ByEvaluator(ByRoleArgs),

    /// List currently active jobs with optional client/provider filter
    Active(ActiveArgs),

    /// List jobs filtered by state (paginated)
    ByState(ByStateArgs),

    /// Get the current job module parameters
    Params,
}

#[derive(Args)]
pub struct GetArgs {
    /// Job ID
    #[arg(required = true)]
    pub job_id: String,
}

#[derive(Args)]
pub struct ByRoleArgs {
    /// Agent hash for the role (client, provider, or evaluator)
    #[arg(required = true)]
    pub agent_hash: String,

    /// Optional state filter
    #[arg(long, value_parser = parse_job_state)]
    pub state: Option<JobState>,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct ActiveArgs {
    /// Filter by client agent hash
    #[arg(long)]
    pub client: Option<String>,

    /// Filter by provider agent hash
    #[arg(long)]
    pub provider: Option<String>,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Args)]
pub struct ByStateArgs {
    /// Job state to filter by
    #[arg(required = true, value_parser = parse_job_state)]
    pub state: JobState,

    #[arg(long, default_value = "20")]
    pub limit: u32,

    #[arg(long, default_value = "0")]
    pub offset: u32,
}

pub async fn execute(cmd: JobQueryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        JobQueryCommands::Get(args) => query_job(args, &dispatcher).await,
        JobQueryCommands::ByClient(args) => query_by_client(args, &dispatcher).await,
        JobQueryCommands::ByProvider(args) => query_by_provider(args, &dispatcher).await,
        JobQueryCommands::ByEvaluator(args) => query_by_evaluator(args, &dispatcher).await,
        JobQueryCommands::Active(args) => query_active(args, &dispatcher).await,
        JobQueryCommands::ByState(args) => query_by_state(args, &dispatcher).await,
        JobQueryCommands::Params => query_params(&dispatcher).await,
    }
}

async fn query_job(args: GetArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_job(args.job_id).await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_client(args: ByRoleArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_jobs_by_client(args.agent_hash, args.state, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_provider(args: ByRoleArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_jobs_by_provider(args.agent_hash, args.state, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_evaluator(args: ByRoleArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_jobs_by_evaluator(args.agent_hash, args.state, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active(args: ActiveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_active_jobs(args.client, args.provider, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_state(args: ByStateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client
        .query_jobs_by_state(args.state, args.limit, args.offset)
        .await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let transport = dispatcher.grpc_transport().await?;
    let client = morpheum_sdk_native::job::JobClient::new(
        dispatcher.sdk_config(),
        Box::new(transport),
    );
    let result = client.query_params().await?;
    let json = serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{result:?}"));
    println!("{json}");
    Ok(())
}

fn parse_job_state(s: &str) -> Result<JobState, String> {
    use JobState::*;
    match s.to_lowercase().as_str() {
        "open" => Ok(Open),
        "funded" => Ok(Funded),
        "submitted" => Ok(Submitted),
        "completed" => Ok(Completed),
        "rejected" => Ok(Rejected),
        "expired" => Ok(Expired),
        "cancelled" => Ok(Cancelled),
        other => Err(format!(
            "unknown job state '{other}'; expected: open, funded, submitted, completed, rejected, expired, cancelled"
        )),
    }
}
