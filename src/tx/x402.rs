use clap::{Args, Subcommand};

use morpheum_sdk_native::x402::{
    RegisterPolicyBuilder, UpdatePolicyBuilder, RotateAddressBuilder,
    ApproveOutboundBuilder, Policy, Scheme,
};
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the x402 autonomous payment module.
///
/// Manages spending policies, payment address rotation, and outbound
/// payment approval for AI agents using the x402 protocol (HTTP 402 +
/// signed payment requests with TEE-attested settlement).
#[derive(Subcommand)]
pub enum X402Commands {
    /// Register a new spending policy for an agent
    RegisterPolicy(RegisterPolicyArgs),

    /// Update an existing spending policy
    UpdatePolicy(UpdatePolicyArgs),

    /// Rotate the agent's payment address
    RotateAddress(RotateAddressArgs),

    /// Approve an outbound x402 payment
    ApproveOutbound(ApproveOutboundArgs),
}

#[derive(Args)]
pub struct RegisterPolicyArgs {
    /// Agent ID the policy applies to
    #[arg(long)]
    pub agent_id: String,

    /// Maximum spend per service call in USD (micro-precision)
    #[arg(long)]
    pub max_per_service_usd: u64,

    /// Daily spending cap in USD
    #[arg(long)]
    pub daily_cap_usd: u64,

    /// Hourly spending cap in USD
    #[arg(long)]
    pub hourly_cap_usd: u64,

    /// Reputation multiplier in basis points (10000 = 1x)
    #[arg(long, default_value = "10000")]
    pub reputation_multiplier_bps: u32,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct UpdatePolicyArgs {
    /// Policy ID to update
    #[arg(long)]
    pub policy_id: String,

    /// Agent ID the policy applies to
    #[arg(long)]
    pub agent_id: String,

    /// Maximum spend per service call in USD
    #[arg(long)]
    pub max_per_service_usd: u64,

    /// Daily spending cap in USD
    #[arg(long)]
    pub daily_cap_usd: u64,

    /// Hourly spending cap in USD
    #[arg(long)]
    pub hourly_cap_usd: u64,

    /// Reputation multiplier in basis points (10000 = 1x)
    #[arg(long, default_value = "10000")]
    pub reputation_multiplier_bps: u32,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct RotateAddressArgs {
    /// Reason for the rotation
    #[arg(long)]
    pub reason: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct ApproveOutboundArgs {
    /// Agent ID initiating the payment
    #[arg(long)]
    pub agent_id: String,

    /// Destination address or agent hash
    #[arg(long)]
    pub destination: String,

    /// Amount in USD (micro-precision)
    #[arg(long)]
    pub amount: u64,

    /// Asset identifier (e.g. "USDC", "eip155:8453/erc20:0x...")
    #[arg(long)]
    pub asset: String,

    /// Payment scheme (exact, exact-evm)
    #[arg(long, value_parser = parse_scheme, default_value = "exact")]
    pub scheme: Scheme,

    /// Hex-encoded idempotency key
    #[arg(long)]
    pub idempotency_key: String,

    /// Optional payment memo
    #[arg(long)]
    pub payment_memo: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional transaction memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: X402Commands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        X402Commands::RegisterPolicy(args) => register_policy(args, &dispatcher).await,
        X402Commands::UpdatePolicy(args) => update_policy(args, &dispatcher).await,
        X402Commands::RotateAddress(args) => rotate_address(args, &dispatcher).await,
        X402Commands::ApproveOutbound(args) => approve_outbound(args, &dispatcher).await,
    }
}

async fn register_policy(
    args: RegisterPolicyArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let policy = Policy {
        policy_id: String::new(),
        agent_id: args.agent_id.clone(),
        max_per_service_usd: args.max_per_service_usd,
        daily_cap_usd: args.daily_cap_usd,
        hourly_cap_usd: args.hourly_cap_usd,
        reputation_multiplier_bps: args.reputation_multiplier_bps,
        last_updated: 0,
    };

    let request = RegisterPolicyBuilder::new()
        .owner_address(morpheum_sdk_native::AccountId::new(signer.account_id().0))
        .policy(policy)
        .owner_signature(owner_sig)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "x402 policy registered for agent {}\nDaily cap: ${}, Hourly cap: ${}\nTxHash: {}",
        args.agent_id, args.daily_cap_usd, args.hourly_cap_usd, txhash,
    ));

    Ok(())
}

async fn update_policy(
    args: UpdatePolicyArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let updated_policy = Policy {
        policy_id: args.policy_id.clone(),
        agent_id: args.agent_id.clone(),
        max_per_service_usd: args.max_per_service_usd,
        daily_cap_usd: args.daily_cap_usd,
        hourly_cap_usd: args.hourly_cap_usd,
        reputation_multiplier_bps: args.reputation_multiplier_bps,
        last_updated: 0,
    };

    let request = UpdatePolicyBuilder::new()
        .owner_address(morpheum_sdk_native::AccountId::new(signer.account_id().0))
        .policy_id(&args.policy_id)
        .updated_policy(updated_policy)
        .owner_signature(owner_sig)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "x402 policy {} updated for agent {}\nTxHash: {}",
        args.policy_id, args.agent_id, txhash,
    ));

    Ok(())
}

async fn rotate_address(
    args: RotateAddressArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let mut builder = RotateAddressBuilder::new()
        .owner_address(morpheum_sdk_native::AccountId::new(signer.account_id().0))
        .owner_signature(owner_sig);

    if let Some(ref reason) = args.reason {
        builder = builder.reason(reason);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "x402 payment address rotated\nReason: {}\nTxHash: {}",
        args.reason.as_deref().unwrap_or("unspecified"),
        txhash,
    ));

    Ok(())
}

async fn approve_outbound(
    args: ApproveOutboundArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let idem_key = hex::decode(&args.idempotency_key).map_err(|e| {
        CliError::invalid_input(format!("invalid hex idempotency key: {e}"))
    })?;

    let mut builder = ApproveOutboundBuilder::new()
        .agent_id(&args.agent_id)
        .destination(&args.destination)
        .amount(args.amount)
        .asset(&args.asset)
        .scheme(args.scheme)
        .idempotency_key(idem_key);

    if let Some(ref memo) = args.payment_memo {
        builder = builder.memo(memo);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "x402 outbound payment approved\nAgent: {}, Destination: {}\nAmount: {} {}\nTxHash: {}",
        args.agent_id, args.destination, args.amount, args.asset, txhash,
    ));

    Ok(())
}

fn parse_scheme(s: &str) -> Result<Scheme, String> {
    match s.to_lowercase().as_str() {
        "exact" => Ok(Scheme::Exact),
        "exact-evm" | "evm" => Ok(Scheme::ExactEvm),
        other => Err(format!(
            "unknown scheme '{other}'; expected: exact, exact-evm"
        )),
    }
}
