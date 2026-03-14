use clap::{Args, Subcommand};

use morpheum_sdk_native::MorpheumSdk;
use morpheum_sdk_native::job::types::JobState;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::output::Output;
use crate::utils::QueryClientExt;

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
    let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);

    match cmd {
        JobQueryCommands::Get(args) => query_job(args, &sdk, &dispatcher.output).await,
        JobQueryCommands::ByClient(args) => {
            query_by_client(args, &sdk, &dispatcher.output).await
        }
        JobQueryCommands::ByProvider(args) => {
            query_by_provider(args, &sdk, &dispatcher.output).await
        }
        JobQueryCommands::ByEvaluator(args) => {
            query_by_evaluator(args, &sdk, &dispatcher.output).await
        }
        JobQueryCommands::Active(args) => {
            query_active(args, &sdk, &dispatcher.output).await
        }
        JobQueryCommands::ByState(args) => {
            query_by_state(args, &sdk, &dispatcher.output).await
        }
        JobQueryCommands::Params => query_params(&sdk, &dispatcher.output).await,
    }
}

async fn query_job(
    args: GetArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_optional(
            output,
            &format!("No job found with ID {}", args.job_id),
            |c| async move { c.query_job(&args.job_id).await },
        )
        .await
}

async fn query_by_client(
    args: ByRoleArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_list(output, |c| async move {
            c.query_jobs_by_client(&args.agent_hash, args.state, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_by_provider(
    args: ByRoleArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_list(output, |c| async move {
            c.query_jobs_by_provider(&args.agent_hash, args.state, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_by_evaluator(
    args: ByRoleArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_list(output, |c| async move {
            c.query_jobs_by_evaluator(&args.agent_hash, args.state, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_active(
    args: ActiveArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_list(output, |c| async move {
            c.query_active_jobs(args.client, args.provider, args.limit, args.offset)
                .await
        })
        .await
}

async fn query_by_state(
    args: ByStateArgs,
    sdk: &MorpheumSdk,
    output: &Output,
) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_list(output, |c| async move {
            c.query_jobs_by_state(args.state, args.limit, args.offset)
                .await
        })
        .await
}

/// `query_params` returns `Result<JobParams, SdkError>` (non-Optional).
async fn query_params(sdk: &MorpheumSdk, output: &Output) -> Result<(), CliError> {
    sdk.job()
        .query_and_print_item(output, |c| async move { c.query_params().await })
        .await
}

fn parse_job_state(s: &str) -> Result<JobState, String> {
    match s.to_lowercase().as_str() {
        "created" => Ok(JobState::Created),
        "funded" => Ok(JobState::Funded),
        "active" => Ok(JobState::Active),
        "delivered" => Ok(JobState::Delivered),
        "attested" => Ok(JobState::Attested),
        "completed" => Ok(JobState::Completed),
        "disputed" => Ok(JobState::Disputed),
        "cancelled" => Ok(JobState::Cancelled),
        "refunded" => Ok(JobState::Refunded),
        other => Err(format!(
            "unknown job state '{other}'; expected: created, funded, active, delivered, \
             attested, completed, disputed, cancelled, refunded"
        )),
    }
}
