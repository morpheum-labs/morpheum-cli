use clap::{Args, Subcommand, ValueEnum};

use morpheum_sdk_native::x402::{
    RegisterPolicyBuilder, UpdatePolicyBuilder, RotateAddressBuilder,
    ApproveOutboundBuilder, SettleBridgePaymentBuilder, Policy, Scheme,
    resolve_chain_name,
};
use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the x402 autonomous payment module.
///
/// Manages spending policies, payment address rotation, outbound
/// payment approval, and cross-chain x402 payment execution for AI agents
/// using the x402 protocol (HTTP 402 + signed payment requests with
/// TEE-attested settlement).
#[derive(Subcommand)]
pub enum X402Commands {
    /// Pay a Morpheum agent from an external chain (EVM/SVM)
    Pay(PayArgs),

    /// Register a new spending policy for an agent
    RegisterPolicy(RegisterPolicyArgs),

    /// Update an existing spending policy
    UpdatePolicy(UpdatePolicyArgs),

    /// Rotate the agent's payment address
    RotateAddress(RotateAddressArgs),

    /// Approve an outbound x402 payment
    ApproveOutbound(ApproveOutboundArgs),

    /// Settle a cross-chain bridge payment (relay/operator)
    SettleBridgePayment(SettleBridgePaymentArgs),
}

/// Supported chain types for x402 payment.
#[derive(Clone, Debug, ValueEnum)]
pub enum X402ChainType {
    Evm,
    Svm,
}

#[derive(Args)]
pub struct PayArgs {
    /// Chain type for the source-chain payment
    #[arg(long, value_enum)]
    pub chain: X402ChainType,

    /// Specific chain name (e.g. "ethereum", "base", "solana")
    #[arg(long)]
    pub chain_name: Option<String>,

    /// Morpheum agent ID (hex string) to pay
    #[arg(long)]
    pub agent: String,

    /// Payment amount in human-readable units (e.g. "50" for 50 USDC)
    #[arg(long)]
    pub amount: String,

    /// Token symbol (default: USDC)
    #[arg(long, default_value = "USDC")]
    pub token: String,

    /// Service-specific memo
    #[arg(long, default_value = "")]
    pub memo: String,

    /// GMP reply channel
    #[arg(long, default_value = "hyperlane")]
    pub reply_channel: String,

    /// Hex-encoded payment ID (optional; auto-generated if omitted)
    #[arg(long)]
    pub payment_id: Option<String>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
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

    /// Target chain for payment routing (e.g. "ethereum", "solana", "base").
    /// When set, the scheme is auto-selected (exact-evm for EVM chains, exact-svm for SVM).
    #[arg(long)]
    pub chain: Option<String>,

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

#[derive(Args)]
pub struct SettleBridgePaymentArgs {
    /// Payment ID from the source chain escrow
    #[arg(long)]
    pub payment_id: String,

    /// Source chain in CAIP-2 format (e.g. "eip155:8453" for Base).
    /// Either --source-chain or --chain must be provided.
    #[arg(long)]
    pub source_chain: Option<String>,

    /// Human-readable chain name (e.g. "base", "ethereum", "solana").
    /// Resolved to a CAIP-2 identifier via the SDK chain registry.
    /// Either --chain or --source-chain must be provided.
    #[arg(long)]
    pub chain: Option<String>,

    /// Target agent ID on Morpheum
    #[arg(long)]
    pub target_agent_id: String,

    /// Payment amount in micro-USD
    #[arg(long)]
    pub amount: u64,

    /// Asset identifier (e.g. "USDC")
    #[arg(long)]
    pub asset: String,

    /// Hex-encoded signature payload from the source chain transaction
    #[arg(long)]
    pub signature_payload: String,

    /// GMP reply channel ID
    #[arg(long)]
    pub reply_channel: String,

    /// Optional payment memo
    #[arg(long)]
    pub payment_memo: Option<String>,

    /// EVM address of the original payer on the source chain
    #[arg(long, default_value = "")]
    pub payer_address: String,

    /// Key name to sign with (relayer identity)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional transaction memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: X402Commands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        X402Commands::Pay(args) => pay(args, &dispatcher).await,
        X402Commands::RegisterPolicy(args) => register_policy(args, &dispatcher).await,
        X402Commands::UpdatePolicy(args) => update_policy(args, &dispatcher).await,
        X402Commands::RotateAddress(args) => rotate_address(args, &dispatcher).await,
        X402Commands::ApproveOutbound(args) => approve_outbound(args, &dispatcher).await,
        X402Commands::SettleBridgePayment(args) => settle_bridge_payment(args, &dispatcher).await,
    }
}

async fn pay(args: PayArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    match args.chain {
        X402ChainType::Evm => pay_evm(args, dispatcher).await,
        X402ChainType::Svm => pay_svm(args, dispatcher).await,
    }
}

async fn pay_evm(args: PayArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_sdk_evm::alloy::primitives::{FixedBytes, U256};
    use morpheum_sdk_evm::config::ChainRegistry;
    use morpheum_sdk_core::ChainRegistryOps as _;

    let chain_name = args.chain_name.as_deref().unwrap_or("ethereum");

    let registry = ChainRegistry::load_with_defaults(morpheum_sdk_evm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("EVM", format!("chain registry: {e}")))?;

    let (chain, token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::chain("EVM", format!("resolving chain '{chain_name}': {e}")))?;

    let settlement = token.settlement_contract.ok_or_else(|| {
        CliError::chain("EVM", format!("no settlement contract configured for {} on {chain_name}", args.token))
    })?;

    let alloy_signer = dispatcher.keyring.get_evm_signer(&args.from)?;
    let from_address = format!("{:#x}", morpheum_sdk_evm::alloy::signers::Signer::address(&alloy_signer));

    let agent_bytes = hex::decode(args.agent.strip_prefix("0x").unwrap_or(&args.agent))
        .map_err(|e| CliError::invalid_input(format!("invalid agent hex: {e}")))?;
    let mut agent_id = [0u8; 32];
    if agent_bytes.len() <= 32 {
        agent_id[32 - agent_bytes.len()..].copy_from_slice(&agent_bytes);
    } else {
        return Err(CliError::invalid_input("agent ID must be <= 32 bytes"));
    }

    let payment_id_bytes = match &args.payment_id {
        Some(hex_str) => {
            let decoded = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
                .map_err(|e| CliError::invalid_input(format!("invalid payment_id hex: {e}")))?;
            let mut buf = [0u8; 32];
            if decoded.len() != 32 {
                return Err(CliError::invalid_input("payment_id must be exactly 32 bytes"));
            }
            buf.copy_from_slice(&decoded);
            buf
        }
        None => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
            let mut buf = [0u8; 32];
            buf[..16].copy_from_slice(&ts.to_le_bytes());
            buf[16..].copy_from_slice(&agent_id[..16]);
            buf
        }
    };

    let amount_parts: Vec<&str> = args.amount.split('.').collect();
    let (whole, frac) = match amount_parts.len() {
        1 => (amount_parts[0], ""),
        2 => (amount_parts[0], amount_parts[1]),
        _ => return Err(CliError::invalid_input("invalid amount format")),
    };
    let whole_val: u128 = whole.parse().map_err(|e| CliError::invalid_input(format!("invalid amount: {e}")))?;
    let frac_len = frac.len();
    let frac_val: u128 = if frac.is_empty() { 0 } else {
        frac.parse().map_err(|e| CliError::invalid_input(format!("invalid amount frac: {e}")))?
    };
    let dec = token.decimals as u32;
    let scale = 10u128.pow(dec);
    let frac_scale = 10u128.pow(dec.saturating_sub(frac_len as u32));
    let raw_amount = U256::from(whole_val * scale + frac_val * frac_scale);

    dispatcher.output.info(format!(
        "x402 payment (EVM)\n\
         From: {from_address}\n\
         Chain: {chain_name}\n\
         Settlement: {settlement:#x}\n\
         Agent: 0x{}\n\
         Amount: {} {} ({raw_amount} raw)\n\
         Memo: {}",
        hex::encode(agent_id), args.amount, args.token, args.memo,
    ));

    let provider = morpheum_sdk_evm::build_provider(&chain.rpc_url, alloy_signer.clone())
        .map_err(|e| CliError::chain("EVM", format!("provider: {e}")))?;

    dispatcher.output.info("Approving USDC spend for settlement contract...");
    morpheum_sdk_evm::approve_erc20(&provider, token.address, settlement, raw_amount)
        .await
        .map_err(|e| CliError::chain("EVM", format!("approve: {e}")))?;

    dispatcher.output.info("Calling pay()...");
    let params = morpheum_sdk_evm::X402PayParams {
        payment_id: FixedBytes(payment_id_bytes),
        target_agent_id: FixedBytes(agent_id),
        amount: raw_amount,
        memo: args.memo,
        reply_channel: args.reply_channel,
        msg_value: U256::ZERO,
    };

    let result = morpheum_sdk_evm::pay_x402(&provider, settlement, &alloy_signer, &params)
        .await
        .map_err(|e| CliError::chain("EVM", format!("pay: {e}")))?;

    dispatcher.output.success(format!(
        "x402 payment submitted (EVM)\n\
         TxHash: {:#x}\n\
         PaymentID: {:#x}\n\
         Amount: {} {}",
        result.tx_hash, result.payment_id, args.amount, args.token,
    ));

    Ok(())
}

async fn pay_svm(args: PayArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_sdk_svm::solana_sdk::signer::keypair::Keypair;
    use morpheum_sdk_svm::config::SolanaChainRegistry;
    use morpheum_sdk_core::ChainRegistryOps as _;

    let chain_name = args.chain_name.as_deref().unwrap_or("solana");

    let registry = SolanaChainRegistry::load_with_defaults(morpheum_sdk_svm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("SVM", format!("chain registry: {e}")))?;

    let (chain, token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::chain("SVM", format!("resolving chain '{chain_name}': {e}")))?;

    let solana_signer = dispatcher.keyring.get_solana_signer(&args.from)?;
    let from_address = bs58::encode(solana_signer.public_key_bytes()).into_string();

    let agent_bytes = hex::decode(args.agent.strip_prefix("0x").unwrap_or(&args.agent))
        .map_err(|e| CliError::invalid_input(format!("invalid agent hex: {e}")))?;
    let mut agent_id = [0u8; 32];
    if agent_bytes.len() <= 32 {
        agent_id[32 - agent_bytes.len()..].copy_from_slice(&agent_bytes);
    } else {
        return Err(CliError::invalid_input("agent ID must be <= 32 bytes"));
    }

    let payment_id = match &args.payment_id {
        Some(hex_str) => {
            let decoded = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
                .map_err(|e| CliError::invalid_input(format!("invalid payment_id hex: {e}")))?;
            let mut buf = [0u8; 32];
            if decoded.len() != 32 {
                return Err(CliError::invalid_input("payment_id must be exactly 32 bytes"));
            }
            buf.copy_from_slice(&decoded);
            buf
        }
        None => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
            let mut buf = [0u8; 32];
            buf[..16].copy_from_slice(&ts.to_le_bytes());
            buf[16..].copy_from_slice(&agent_id[..16]);
            buf
        }
    };

    let amount: u64 = args.amount.parse()
        .map_err(|e| CliError::invalid_input(format!("invalid amount: {e}")))?;

    dispatcher.output.info(format!(
        "x402 payment (SVM)\n\
         From: {from_address}\n\
         Chain: {chain_name}\n\
         Token: {} (mint: {})\n\
         Agent: 0x{}\n\
         Amount: {amount}",
        args.token, token.mint, hex::encode(agent_id),
    ));

    let mut keypair_bytes = [0u8; 64];
    keypair_bytes[..32].copy_from_slice(&solana_signer.private_key_bytes());
    keypair_bytes[32..].copy_from_slice(&solana_signer.public_key_bytes());
    let keypair = Keypair::try_from(keypair_bytes.as_slice())
        .map_err(|e| CliError::chain("SVM", format!("keypair: {e}")))?;

    let provider = morpheum_sdk_svm::provider::build_provider(&chain.rpc_url, keypair)
        .map_err(|e| CliError::chain("SVM", format!("provider: {e}")))?;

    dispatcher.output.info("Calling pay_x402...");
    let result = morpheum_sdk_svm::x402::pay_x402(
        &provider,
        &token.mint,
        payment_id,
        agent_id,
        amount,
        &args.reply_channel,
    )
    .map_err(|e| CliError::chain("SVM", format!("pay_x402: {e}")))?;

    dispatcher.output.success(format!(
        "x402 payment submitted (SVM)\n\
         Signature: {}\n\
         PaymentID: 0x{}\n\
         Amount: {amount} {}",
        result.signature, hex::encode(result.payment_id), args.token,
    ));

    Ok(())
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

    let scheme = resolve_payment_scheme(&args)?;

    let mut builder = ApproveOutboundBuilder::new()
        .agent_id(&args.agent_id)
        .destination(&args.destination)
        .amount(args.amount)
        .asset(&args.asset)
        .scheme(scheme)
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

    let chain_label = args.chain.as_deref().unwrap_or("native");
    dispatcher.output.success(format!(
        "x402 outbound payment approved ({chain_label})\nAgent: {}, Destination: {}\nAmount: {} {}\nTxHash: {}",
        args.agent_id, args.destination, args.amount, args.asset, txhash,
    ));

    Ok(())
}

/// Resolves the payment scheme from `--chain` (if given) or falls back to `--scheme`.
fn resolve_payment_scheme(args: &ApproveOutboundArgs) -> Result<Scheme, CliError> {
    if let Some(ref chain_name) = args.chain {
        let meta = resolve_chain_name(chain_name).ok_or_else(|| {
            CliError::invalid_input(format!(
                "unknown chain '{chain_name}'; use --scheme explicitly or provide a known chain name"
            ))
        })?;
        let caip2 = meta.caip2();
        if caip2.starts_with("eip155:") {
            Ok(Scheme::ExactEvm)
        } else if caip2.starts_with("solana:") {
            Ok(Scheme::ExactSvm)
        } else {
            Ok(Scheme::Exact)
        }
    } else {
        Ok(args.scheme.clone())
    }
}

async fn settle_bridge_payment(
    args: SettleBridgePaymentArgs,
    dispatcher: &Dispatcher,
) -> Result<(), CliError> {
    let source_chain = resolve_source_chain(&args)?;

    let signer = dispatcher.keyring.get_native_signer(&args.from)?;

    let sig_payload = hex::decode(&args.signature_payload).map_err(|e| {
        CliError::invalid_input(format!("invalid hex signature_payload: {e}"))
    })?;

    let mut builder = SettleBridgePaymentBuilder::new()
        .relayer_address(hex::encode(signer.account_id().0))
        .payment_id(&args.payment_id)
        .source_chain(&source_chain)
        .target_agent_id(&args.target_agent_id)
        .amount(args.amount)
        .asset(&args.asset)
        .signature_payload(sig_payload)
        .reply_channel(&args.reply_channel)
        .payer_address(&args.payer_address);

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
        "x402 bridge settlement submitted\n\
         Payment ID: {}\n\
         Source: {} → Agent: {}\n\
         Amount: {} {}\n\
         TxHash: {}",
        args.payment_id, source_chain, args.target_agent_id,
        args.amount, args.asset, txhash,
    ));

    Ok(())
}

/// Resolves `--chain` or `--source-chain` into a CAIP-2 source chain string.
fn resolve_source_chain(args: &SettleBridgePaymentArgs) -> Result<String, CliError> {
    match (&args.source_chain, &args.chain) {
        (Some(sc), _) => Ok(sc.clone()),
        (None, Some(name)) => {
            let meta = resolve_chain_name(name).ok_or_else(|| {
                CliError::invalid_input(format!(
                    "unknown chain name '{name}'; use --source-chain with a CAIP-2 identifier instead"
                ))
            })?;
            Ok(meta.caip2())
        }
        (None, None) => Err(CliError::invalid_input(
            "either --source-chain (CAIP-2) or --chain (name) is required",
        )),
    }
}

fn parse_scheme(s: &str) -> Result<Scheme, String> {
    match s.to_lowercase().as_str() {
        "exact" => Ok(Scheme::Exact),
        "exact-evm" | "evm" => Ok(Scheme::ExactEvm),
        "exact-svm" | "svm" => Ok(Scheme::ExactSvm),
        other => Err(format!(
            "unknown scheme '{other}'; expected: exact, exact-evm, exact-svm"
        )),
    }
}
