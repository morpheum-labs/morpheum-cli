use clap::{Args, Subcommand};

use morpheum_sdk_gov::builder::{
    CancelProposalBuilder, ProposalDepositBuilder, ProposalVoteBuilder,
    ScheduleUpgradeBuilder, SubmitProposalBuilder,
};
use morpheum_sdk_gov::types::{
    ProposalClass, UpgradePlan, VoteOption, WeightedVoteOption,
};
use morpheum_sdk_gov::AccountId;
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `gov` module.
///
/// Covers proposal submission, deposits, voting (weighted split + conviction),
/// proposal cancellation, and zero-downtime upgrade scheduling.
#[derive(Subcommand)]
pub enum GovCommands {
    /// Submit a new governance proposal
    SubmitProposal(SubmitProposalArgs),

    /// Add a deposit to an existing proposal
    Deposit(DepositArgs),

    /// Cast a vote on a proposal (supports weighted split + conviction)
    Vote(VoteArgs),

    /// Cancel a proposal (proposer only, during deposit period)
    CancelProposal(CancelProposalArgs),

    /// Schedule a zero-downtime software upgrade via governance
    ScheduleUpgrade(ScheduleUpgradeArgs),
}

#[derive(Args)]
pub struct SubmitProposalArgs {
    /// Proposal title
    #[arg(long)]
    pub title: String,

    /// Proposal description
    #[arg(long)]
    pub description: String,

    /// Proposal class (standard, expedited, emergency, root, market, treasury, emergency-market)
    #[arg(long, value_parser = parse_proposal_class)]
    pub class: ProposalClass,

    /// Initial deposit amount (e.g. "1000000")
    #[arg(long)]
    pub deposit: String,

    /// Optional metadata URI (e.g. `ipfs://Qm...`)
    #[arg(long, default_value = "")]
    pub metadata: String,

    /// Execution messages as JSON: `[{"type_url":"/mod.v1.Msg","value":"<hex>"}]`
    ///
    /// Each element's `value` is the hex-encoded protobuf-serialized message body.
    /// These messages are executed atomically when the proposal passes.
    #[arg(long, default_value = "")]
    pub messages: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct DepositArgs {
    /// Proposal ID to deposit on
    #[arg(long)]
    pub proposal_id: u64,

    /// Deposit amount
    #[arg(long)]
    pub amount: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct VoteArgs {
    /// Proposal ID to vote on
    #[arg(long)]
    pub proposal_id: u64,

    /// Vote option (yes, no, abstain, no-with-veto)
    #[arg(long, value_parser = parse_vote_option)]
    pub option: VoteOption,

    /// Vote weight (e.g. "1.0" for full weight; supports split voting)
    #[arg(long, default_value = "1.0")]
    pub weight: String,

    /// Conviction multiplier (0 = no lock, 1..6 = increasing lock duration)
    #[arg(long, default_value = "0")]
    pub conviction: u64,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct CancelProposalArgs {
    /// Proposal ID to cancel
    #[arg(long)]
    pub proposal_id: u64,

    /// Reason for cancellation
    #[arg(long, default_value = "")]
    pub reason: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct ScheduleUpgradeArgs {
    /// Upgrade proposal title
    #[arg(long)]
    pub title: String,

    /// Upgrade proposal description
    #[arg(long)]
    pub description: String,

    /// Proposal class (typically root or emergency)
    #[arg(long, value_parser = parse_proposal_class)]
    pub class: ProposalClass,

    /// Upgrade plan name (e.g. "v2.1.0-morpheum")
    #[arg(long)]
    pub name: String,

    /// Upgrade info URI (e.g. `ipfs://QmUpgrade...`)
    #[arg(long, default_value = "")]
    pub info: String,

    /// Grace period in seconds before forced activation
    #[arg(long, default_value = "3600")]
    pub grace_period: u64,

    /// Initial deposit amount
    #[arg(long)]
    pub deposit: String,

    /// Optional metadata URI
    #[arg(long, default_value = "")]
    pub metadata: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

pub async fn execute(cmd: GovCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        GovCommands::SubmitProposal(args) => submit_proposal(args, &dispatcher).await,
        GovCommands::Deposit(args) => deposit(args, &dispatcher).await,
        GovCommands::Vote(args) => vote(args, &dispatcher).await,
        GovCommands::CancelProposal(args) => cancel_proposal(args, &dispatcher).await,
        GovCommands::ScheduleUpgrade(args) => schedule_upgrade(args, &dispatcher).await,
    }
}

async fn submit_proposal(
    args: SubmitProposalArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let account_id = AccountId::from(signer.account_id());

    let mut builder = SubmitProposalBuilder::new()
        .from_address(account_id)
        .proposal_class(args.class)
        .title(&args.title)
        .description(&args.description)
        .initial_deposit(&args.deposit);

    if !args.metadata.is_empty() {
        builder = builder.metadata(&args.metadata);
    }

    if !args.messages.is_empty() {
        let msgs = parse_execution_messages(&args.messages)?;
        builder = builder.messages(msgs);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Proposal submitted\nTitle: {}\nClass: {:?}\nDeposit: {}\nTxHash: {}",
        args.title, args.class, args.deposit, txhash,
    ));

    Ok(())
}

/// Parses a JSON array of execution messages into `Vec<ProtoAny>`.
///
/// Expected format: `[{"type_url":"/mod.v1.Msg","value":"<hex>"}]`
fn parse_execution_messages(json_str: &str) -> Result<Vec<morpheum_proto::google::protobuf::Any>, CliError> {
    #[derive(serde::Deserialize)]
    struct RawMessage {
        type_url: String,
        value: String,
    }

    let raw: Vec<RawMessage> = serde_json::from_str(json_str)
        .map_err(|e| CliError::InvalidInput { reason: format!("invalid --messages JSON: {e}") })?;

    raw.into_iter()
        .map(|m| {
            let bytes = hex::decode(&m.value)
                .map_err(|e| CliError::InvalidInput {
                    reason: format!("invalid hex in message value for '{}': {e}", m.type_url),
                })?;
            Ok(morpheum_proto::google::protobuf::Any {
                type_url: m.type_url,
                value: bytes,
            })
        })
        .collect()
}

async fn deposit(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let account_id = AccountId::from(signer.account_id());

    let request = ProposalDepositBuilder::new()
        .from_address(account_id)
        .proposal_id(args.proposal_id)
        .amount(&args.amount)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Deposit added\nProposal: {}\nAmount: {}\nTxHash: {}",
        args.proposal_id, args.amount, txhash,
    ));

    Ok(())
}

async fn vote(args: VoteArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let account_id = AccountId::from(signer.account_id());

    let request = ProposalVoteBuilder::new()
        .from_address(account_id)
        .proposal_id(args.proposal_id)
        .add_option(WeightedVoteOption::new(args.option, &args.weight))
        .conviction_multiplier(args.conviction)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Vote cast\nProposal: {}\nOption: {:?}\nWeight: {}\nTxHash: {}",
        args.proposal_id, args.option, args.weight, txhash,
    ));

    Ok(())
}

async fn cancel_proposal(
    args: CancelProposalArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let account_id = AccountId::from(signer.account_id());

    let mut builder = CancelProposalBuilder::new()
        .from_address(account_id)
        .proposal_id(args.proposal_id);

    if !args.reason.is_empty() {
        builder = builder.reason(&args.reason);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Proposal cancelled\nProposal: {}\nTxHash: {}",
        args.proposal_id, txhash,
    ));

    Ok(())
}

async fn schedule_upgrade(
    args: ScheduleUpgradeArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let account_id = AccountId::from(signer.account_id());

    let plan = UpgradePlan {
        name: args.name.clone(),
        info: args.info,
        activation_staple_id: 0,
        activation_time: 0,
        binary_hash: Vec::new(),
        grace_period_seconds: args.grace_period,
        additional_metadata: std::collections::BTreeMap::new(),
    };

    let mut builder = ScheduleUpgradeBuilder::new()
        .from_address(account_id)
        .proposal_class(args.class)
        .upgrade_plan(plan)
        .title(&args.title)
        .description(&args.description)
        .initial_deposit(&args.deposit);

    if !args.metadata.is_empty() {
        builder = builder.metadata(&args.metadata);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Upgrade scheduled\nName: {}\nTitle: {}\nGrace: {}s\nTxHash: {}",
        args.name, args.title, args.grace_period, txhash,
    ));

    Ok(())
}

fn parse_proposal_class(s: &str) -> Result<ProposalClass, String> {
    match s.to_lowercase().as_str() {
        "standard" => Ok(ProposalClass::Standard),
        "expedited" => Ok(ProposalClass::Expedited),
        "emergency" => Ok(ProposalClass::Emergency),
        "root" => Ok(ProposalClass::Root),
        "market" => Ok(ProposalClass::Market),
        "treasury" => Ok(ProposalClass::Treasury),
        "emergency-market" | "emergencymarket" => Ok(ProposalClass::EmergencyMarket),
        other => Err(format!(
            "unknown proposal class '{other}'; expected: standard, expedited, \
             emergency, root, market, treasury, emergency-market"
        )),
    }
}

fn parse_vote_option(s: &str) -> Result<VoteOption, String> {
    match s.to_lowercase().as_str() {
        "yes" | "y" => Ok(VoteOption::Yes),
        "no" | "n" => Ok(VoteOption::No),
        "abstain" | "a" => Ok(VoteOption::Abstain),
        "no-with-veto" | "nowithveto" | "veto" | "nwv" => Ok(VoteOption::NoWithVeto),
        other => Err(format!(
            "unknown vote option '{other}'; expected: yes, no, abstain, no-with-veto"
        )),
    }
}
