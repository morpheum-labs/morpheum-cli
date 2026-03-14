use clap::{Args, Subcommand};

use morpheum_proto::job::v1::{JobState, QueryJobRequest, QueryJobsByClientRequest,
    QueryJobsByProviderRequest, QueryJobsByEvaluatorRequest, QueryActiveJobsRequest,
    QueryJobsByStateRequest, QueryParamsRequest};
use morpheum_proto::job::v1::query_client::QueryClient;

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
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let job_id = args.job_id.clone();
    let response = client
        .query_job(tonic::Request::new(QueryJobRequest {
            job_id: args.job_id,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryJob failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    if response.found {
        println!("{json}");
    } else {
        println!("No job found with ID {job_id}");
    }
    Ok(())
}

async fn query_by_client(args: ByRoleArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_jobs_by_client(tonic::Request::new(QueryJobsByClientRequest {
            client_agent_hash: args.agent_hash,
            state: args.state.map_or(0, Into::into),
            limit: args.limit,
            offset: args.offset,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryJobsByClient failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_provider(args: ByRoleArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_jobs_by_provider(tonic::Request::new(QueryJobsByProviderRequest {
            provider_agent_hash: args.agent_hash,
            state: args.state.map_or(0, Into::into),
            limit: args.limit,
            offset: args.offset,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryJobsByProvider failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_evaluator(args: ByRoleArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_jobs_by_evaluator(tonic::Request::new(QueryJobsByEvaluatorRequest {
            evaluator_agent_hash: args.agent_hash,
            state: args.state.map_or(0, Into::into),
            limit: args.limit,
            offset: args.offset,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryJobsByEvaluator failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_active(args: ActiveArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_active_jobs(tonic::Request::new(QueryActiveJobsRequest {
            client_agent_hash: args.client.unwrap_or_default(),
            provider_agent_hash: args.provider.unwrap_or_default(),
            limit: args.limit,
            offset: args.offset,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryActiveJobs failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_by_state(args: ByStateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_jobs_by_state(tonic::Request::new(QueryJobsByStateRequest {
            state: args.state.into(),
            limit: args.limit,
            offset: args.offset,
        }))
        .await
        .map_err(|e| CliError::Transport(format!("QueryJobsByState failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

async fn query_params(dispatcher: &Dispatcher) -> Result<(), CliError> {
    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = QueryClient::new(channel);
    let response = client
        .query_params(tonic::Request::new(QueryParamsRequest {}))
        .await
        .map_err(|e| CliError::Transport(format!("QueryParams failed: {e}")))?
        .into_inner();
    let json = serde_json::to_string_pretty(&response).unwrap_or_else(|_| format!("{response:?}"));
    println!("{json}");
    Ok(())
}

fn parse_job_state(s: &str) -> Result<JobState, String> {
    match s.to_lowercase().as_str() {
        "open" => Ok(JobState::Open),
        "funded" => Ok(JobState::Funded),
        "submitted" => Ok(JobState::Submitted),
        "completed" => Ok(JobState::Completed),
        "rejected" => Ok(JobState::Rejected),
        "expired" => Ok(JobState::Expired),
        "cancelled" => Ok(JobState::Cancelled),
        other => Err(format!(
            "unknown job state '{other}'; expected: open, funded, submitted, completed, rejected, expired, cancelled"
        )),
    }
}
