//! Bridge CLI command definitions and handlers.

use clap::{Args, Subcommand, ValueEnum};
use morpheum_sdk_evm::config::ChainRegistry;
use morpheum_sdk_evm::alloy::primitives::{FixedBytes, U256};
use morpheum_sdk_svm::config::SolanaChainRegistry;

use morpheum_signing_native::signer::Signer;

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

use morpheum_sdk_core::ChainRegistryOps as _;

/// Supported external chain types for bridge operations.
#[derive(Clone, Debug, ValueEnum)]
pub enum ChainType {
    /// Ethereum / EVM-compatible chain
    Evm,
    /// Solana / SVM-compatible chain
    Svm,
}

impl ChainType {
    pub fn label(&self) -> &'static str {
        match self {
            ChainType::Evm => "EVM",
            ChainType::Svm => "SVM",
        }
    }
}

/// Bridge protocol to use for the transfer.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum BridgeProtocol {
    /// Hyperlane Warp Route (default, locks & transfers wrapped token)
    #[default]
    WarpRoute,
    /// Circle CCTP via `CctpHyperlaneWrapper` (burns native USDC)
    Cctp,
}

/// Bridge commands for cross-chain token transfers via Hyperlane Warp Routes.
#[derive(Subcommand)]
pub enum BridgeCommands {
    /// Deposit tokens from an external chain (EVM/SVM) to Morpheum
    Deposit(DepositArgs),

    /// Withdraw tokens from Morpheum to an external chain (EVM/SVM)
    Withdraw(WithdrawArgs),

    /// Check the status of a bridge transfer
    Status(StatusArgs),
}

#[derive(Args)]
pub struct DepositArgs {
    /// External chain type
    #[arg(long, value_enum)]
    pub chain: ChainType,

    /// Chain name (e.g. "ethereum", "solana", "arbitrum", "devnet")
    #[arg(long)]
    pub chain_name: Option<String>,

    /// Token symbol to deposit (e.g. "USDC", "WETH")
    #[arg(long)]
    pub token: String,

    /// Amount to deposit
    #[arg(long)]
    pub amount: String,

    /// 32-byte hex recipient address on Morpheum (defaults to sender's Morpheum address)
    #[arg(long)]
    pub recipient: Option<String>,

    /// Morpheum Hyperlane domain ID
    #[arg(long, default_value = "1836016741")]
    pub destination_domain: u32,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Bridge protocol: warp-route (default) or cctp
    #[arg(long, value_enum, default_value = "warp-route")]
    pub protocol: BridgeProtocol,

    /// `CctpHyperlaneWrapper` contract address (required for --protocol cctp)
    #[arg(long)]
    pub wrapper: Option<String>,

    /// Optional hex-encoded calldata for post-mint hook actions (CCTP only)
    #[arg(long)]
    pub calldata: Option<String>,
}

#[derive(Args)]
pub struct WithdrawArgs {
    /// External chain type for the destination
    #[arg(long, value_enum)]
    pub chain: ChainType,

    /// Hyperlane domain ID of the destination chain
    #[arg(long)]
    pub destination_domain: u32,

    /// 32-byte hex recipient address on the destination chain
    #[arg(long)]
    pub recipient: String,

    /// `CosmWasm` Warp Route contract address on Morpheum
    #[arg(long)]
    pub warp_route_contract: String,

    /// Amount to withdraw (in raw token units)
    #[arg(long)]
    pub amount: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Chain type to query
    #[arg(long, value_enum)]
    pub chain: ChainType,

    /// Hyperlane message ID (hex) to check delivery status
    #[arg(long)]
    pub message_id: String,

    /// Bridge protocol used (affects status output)
    #[arg(long, value_enum, default_value = "warp-route")]
    pub protocol: BridgeProtocol,

    /// CCTP handler contract address (for --protocol cctp status queries)
    #[arg(long)]
    pub cctp_handler: Option<String>,
}

pub async fn execute(cmd: BridgeCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        BridgeCommands::Deposit(args) => deposit(args, &dispatcher).await,
        BridgeCommands::Withdraw(args) => withdraw(args, &dispatcher).await,
        BridgeCommands::Status(args) => status(args, &dispatcher).await,
    }
}

// ── Shared helpers ──────────────────────────────────────────────────

fn resolve_recipient(
    explicit: &Option<String>,
    key_name: &str,
    keyring: &crate::keyring::KeyringManager,
    allow_20_byte: bool,
) -> Result<[u8; 32], CliError> {
    let raw = match explicit {
        Some(hex_str) => {
            let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            hex::decode(s)
                .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?
        }
        None => {
            let native = keyring.get_native_signer(key_name)?;
            native.account_id().0.to_vec()
        }
    };

    let mut fixed = [0u8; 32];
    if raw.len() == 32 {
        fixed.copy_from_slice(&raw);
    } else if raw.len() == 20 && allow_20_byte {
        fixed[12..].copy_from_slice(&raw);
    } else if allow_20_byte {
        return Err(CliError::invalid_input("recipient must be 20 or 32 bytes"));
    } else {
        return Err(CliError::invalid_input("recipient must be exactly 32 bytes"));
    }
    Ok(fixed)
}

/// Parses a human-readable amount (e.g. "100") into an on-chain integer
/// using the token's decimal precision (e.g. 6 decimals -> 100_000_000).
fn parse_token_amount(amount_str: &str, decimals: u8) -> Result<U256, CliError> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    let (whole, frac) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err(CliError::invalid_input("invalid amount format")),
    };

    let whole_val: u128 = whole.parse()
        .map_err(|e| CliError::invalid_input(format!("invalid amount: {e}")))?;

    let frac_len = frac.len();
    if frac_len > decimals as usize {
        return Err(CliError::invalid_input(format!(
            "amount has {frac_len} fractional digits but token only supports {decimals}"
        )));
    }

    let frac_val: u128 = if frac.is_empty() {
        0
    } else {
        frac.parse().map_err(|e| CliError::invalid_input(format!("invalid fractional part: {e}")))?
    };

    let scale = 10u128.pow(decimals as u32);
    let frac_scale = 10u128.pow((decimals as u32) - (frac_len as u32));
    let raw = whole_val * scale + frac_val * frac_scale;

    Ok(U256::from(raw))
}

// ── Deposit (External -> Morpheum) ──────────────────────────────────

async fn deposit(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    match (&args.chain, &args.protocol) {
        (ChainType::Evm, BridgeProtocol::Cctp) => deposit_evm_cctp(args, dispatcher).await,
        (ChainType::Evm, BridgeProtocol::WarpRoute) => deposit_evm(args, dispatcher).await,
        (ChainType::Svm, BridgeProtocol::Cctp) => Err(CliError::invalid_input(
            "CCTP protocol is only supported for EVM chains",
        )),
        (ChainType::Svm, BridgeProtocol::WarpRoute) => deposit_svm(args, dispatcher).await,
    }
}

async fn deposit_evm(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let chain_name = args.chain_name.as_deref().unwrap_or("ethereum");

    let registry = ChainRegistry::load_with_defaults(morpheum_sdk_evm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("EVM", format!("chain registry: {e}")))?;

    let (chain, token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::chain("EVM", format!("resolving chain '{chain_name}': {e}")))?;

    let collateral = token.collateral_contract.ok_or_else(|| {
        CliError::chain("EVM", format!("no collateral contract configured for {} on {chain_name}", args.token))
    })?;

    let alloy_signer = dispatcher.keyring.get_evm_signer(&args.from)?;
    let from_address = format!("{:#x}", morpheum_sdk_evm::alloy::signers::Signer::address(&alloy_signer));
    let recipient = resolve_recipient(&args.recipient, &args.from, &dispatcher.keyring, true)?;

    let amount = parse_token_amount(&args.amount, token.decimals)?;

    dispatcher.output.info(format!(
        "EVM bridge deposit\n\
         From: {from_address}\n\
         Chain: {chain_name} (RPC: {})\n\
         Token: {} ({:#x})\n\
         Collateral: {:#x}\n\
         Amount: {} ({} raw)\n\
         Destination domain: {}\n\
         Recipient: 0x{}",
        chain.rpc_url, args.token, token.address, collateral,
        args.amount, amount, args.destination_domain, hex::encode(recipient),
    ));

    let provider = morpheum_sdk_evm::build_provider(&chain.rpc_url, alloy_signer)
        .map_err(|e| CliError::chain("EVM", format!("provider: {e}")))?;

    dispatcher.output.info("Approving ERC-20 spend...");
    let approve_hash = morpheum_sdk_evm::approve_erc20(&provider, token.address, collateral, amount)
        .await
        .map_err(|e| CliError::chain("EVM", format!("approve: {e}")))?;
    dispatcher.output.info(format!("Approval confirmed: {approve_hash:#x}"));

    dispatcher.output.info("Calling transferRemote...");
    let result = morpheum_sdk_evm::transfer_remote(
        &provider,
        collateral,
        args.destination_domain,
        FixedBytes(recipient),
        amount,
        U256::ZERO,
    )
    .await
    .map_err(|e| CliError::chain("EVM", format!("transferRemote: {e}")))?;

    dispatcher.output.success(format!(
        "Bridge deposit submitted (EVM)\n\
         TxHash: {:#x}\n\
         MessageID: {:#x}\n\
         Amount: {} {} -> Morpheum domain {}",
        result.tx_hash, result.message_id,
        args.amount, args.token, args.destination_domain,
    ));

    Ok(())
}

async fn deposit_evm_cctp(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let chain_name = args.chain_name.as_deref().unwrap_or("ethereum");

    let registry = ChainRegistry::load_with_defaults(morpheum_sdk_evm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("EVM", format!("chain registry: {e}")))?;

    let (chain, token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::chain("EVM", format!("resolving chain '{chain_name}': {e}")))?;

    let wrapper_str = args.wrapper.as_deref().ok_or_else(|| {
        CliError::invalid_input("--wrapper is required for CCTP deposits")
    })?;
    let wrapper: morpheum_sdk_evm::alloy::primitives::Address = wrapper_str
        .parse()
        .map_err(|e| CliError::chain("EVM", format!("invalid wrapper address: {e}")))?;

    let alloy_signer = dispatcher.keyring.get_evm_signer(&args.from)?;
    let from_address = format!("{:#x}", morpheum_sdk_evm::alloy::signers::Signer::address(&alloy_signer));
    let recipient = resolve_recipient(&args.recipient, &args.from, &dispatcher.keyring, true)?;

    let amount = parse_token_amount(&args.amount, token.decimals)?;

    let calldata_bytes = match &args.calldata {
        Some(hex_str) => {
            let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            morpheum_sdk_evm::alloy::primitives::Bytes::from(
                hex::decode(s)
                    .map_err(|e| CliError::invalid_input(format!("invalid calldata hex: {e}")))?,
            )
        }
        None => morpheum_sdk_evm::alloy::primitives::Bytes::new(),
    };

    dispatcher.output.info(format!(
        "CCTP bridge deposit\n\
         From: {from_address}\n\
         Chain: {chain_name} (RPC: {})\n\
         Token: {} ({:#x})\n\
         Wrapper: {wrapper:#x}\n\
         Amount: {} ({} raw)\n\
         Destination domain: {}\n\
         Recipient: 0x{}",
        chain.rpc_url, args.token, token.address,
        args.amount, amount, args.destination_domain, hex::encode(recipient),
    ));

    let provider = morpheum_sdk_evm::build_provider(&chain.rpc_url, alloy_signer)
        .map_err(|e| CliError::chain("EVM", format!("provider: {e}")))?;

    dispatcher.output.info("Approving USDC spend for CCTP wrapper...");
    let approve_hash = morpheum_sdk_evm::approve_erc20(&provider, token.address, wrapper, amount)
        .await
        .map_err(|e| CliError::chain("EVM", format!("approve: {e}")))?;
    dispatcher.output.info(format!("Approval confirmed: {approve_hash:#x}"));

    dispatcher.output.info("Quoting Hyperlane dispatch fee...");
    let fee = morpheum_sdk_evm::cctp::quote_cctp_dispatch(
        &provider,
        wrapper,
        amount,
        FixedBytes(recipient),
        calldata_bytes.clone(),
    )
    .await
    .map_err(|e| CliError::chain("EVM", format!("quoteDispatch: {e}")))?;
    dispatcher.output.info(format!("Dispatch fee: {fee} wei"));

    dispatcher.output.info("Calling bridgeUsdc (CCTP burn + Hyperlane dispatch)...");
    let result = morpheum_sdk_evm::cctp::bridge_usdc(
        &provider,
        wrapper,
        amount,
        FixedBytes(recipient),
        calldata_bytes,
        fee,
    )
    .await
    .map_err(|e| CliError::chain("EVM", format!("bridgeUsdc: {e}")))?;

    let nonce_str = result
        .cctp_nonce
        .map_or_else(|| "N/A".to_string(), |n| n.to_string());

    dispatcher.output.success(format!(
        "CCTP bridge deposit submitted\n\
         TxHash: {:#x}\n\
         MessageID: {:#x}\n\
         CCTP Nonce: {nonce_str}\n\
         Amount: {} {} -> Morpheum domain {}",
        result.tx_hash, result.message_id,
        args.amount, args.token, args.destination_domain,
    ));

    Ok(())
}

async fn deposit_svm(args: DepositArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_sdk_svm::solana_sdk::signer::keypair::Keypair;

    let chain_name = args.chain_name.as_deref().unwrap_or("solana");

    let registry = SolanaChainRegistry::load_with_defaults(morpheum_sdk_svm::DEFAULT_CHAINS_TOML)
        .map_err(|e| CliError::chain("SVM", format!("chain registry: {e}")))?;

    let (chain, token) = registry
        .resolve(chain_name, &args.token)
        .map_err(|e| CliError::chain("SVM", format!("resolving chain '{chain_name}': {e}")))?;

    let warp_route = chain.warp_route_program.ok_or_else(|| {
        CliError::chain("SVM", format!("no warp_route_program configured for {chain_name}"))
    })?;

    let solana_signer = dispatcher.keyring.get_solana_signer(&args.from)?;
    let from_address = bs58::encode(solana_signer.public_key_bytes()).into_string();
    let recipient = resolve_recipient(&args.recipient, &args.from, &dispatcher.keyring, false)?;

    let amount: u64 = args.amount.parse()
        .map_err(|e| CliError::invalid_input(format!("invalid amount: {e}")))?;

    dispatcher.output.info(format!(
        "SVM bridge deposit\n\
         From: {from_address}\n\
         Chain: {chain_name} (RPC: {})\n\
         Token: {} (mint: {})\n\
         Warp Route: {warp_route}\n\
         Amount: {amount}\n\
         Destination domain: {}\n\
         Recipient: 0x{}",
        chain.rpc_url, args.token, token.mint,
        args.destination_domain, hex::encode(recipient),
    ));

    let mut keypair_bytes = [0u8; 64];
    keypair_bytes[..32].copy_from_slice(&solana_signer.private_key_bytes());
    keypair_bytes[32..].copy_from_slice(&solana_signer.public_key_bytes());
    let keypair = Keypair::from_bytes(&keypair_bytes)
        .map_err(|e| CliError::chain("SVM", format!("keypair: {e}")))?;

    let provider = morpheum_sdk_svm::provider::build_provider(&chain.rpc_url, keypair)
        .map_err(|e| CliError::chain("SVM", format!("provider: {e}")))?;

    dispatcher.output.info("Calling transfer_remote...");
    let result = morpheum_sdk_svm::bridge::transfer_remote(
        &provider,
        &warp_route,
        &token.mint,
        args.destination_domain,
        recipient,
        amount,
    )
    .map_err(|e| CliError::chain("SVM", format!("transfer_remote: {e}")))?;

    dispatcher.output.success(format!(
        "Bridge deposit submitted (SVM)\n\
         Signature: {}\n\
         MessageID: 0x{}\n\
         Amount: {amount} {} -> Morpheum domain {}",
        result.signature, hex::encode(result.message_id),
        args.token, args.destination_domain,
    ));

    Ok(())
}

// ── Withdraw (Morpheum -> External) ─────────────────────────────────

#[cfg(feature = "_transport")]
async fn withdraw(args: WithdrawArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    use morpheum_sdk_cosmwasm::WarpRouteTransferBuilder;

    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let from_address = hex::encode(signer.account_id().0);

    let recipient_bytes = {
        let s = args.recipient.strip_prefix("0x").unwrap_or(&args.recipient);
        hex::decode(s)
            .map_err(|e| CliError::invalid_input(format!("invalid recipient hex: {e}")))?
    };

    let chain_label = args.chain.label();

    let request = WarpRouteTransferBuilder::new()
        .sender(&from_address)
        .warp_route_contract(&args.warp_route_contract)
        .destination_domain(args.destination_domain)
        .recipient(recipient_bytes)
        .amount(&args.amount)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer,
        dispatcher,
        request.to_any(),
        None,
    )
    .await?;

    dispatcher.output.success(format!(
        "Warp Route withdrawal submitted ({chain_label})\n\
         Contract: {}\n\
         Destination domain: {}\n\
         Amount: {}\n\
         TxHash: {txhash}",
        args.warp_route_contract, args.destination_domain, args.amount,
    ));

    Ok(())
}

#[cfg(not(feature = "_transport"))]
async fn withdraw(_args: WithdrawArgs, _dispatcher: &Dispatcher) -> Result<(), CliError> {
    Err(CliError::invalid_input(
        "withdraw requires transport support — enable the gmp or interop feature",
    ))
}

// ── Status ──────────────────────────────────────────────────────────

async fn status(args: StatusArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let chain_label = args.chain.label();
    let message_id = args.message_id.strip_prefix("0x").unwrap_or(&args.message_id);

    let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
    let mut client = morpheum_proto::gmp::v1::query_client::QueryClient::new(channel);

    let response = client
        .query_hyperlane_delivery(tonic::Request::new(
            morpheum_proto::gmp::v1::QueryHyperlaneDeliveryRequest {
                message_id: message_id.to_string(),
            },
        ))
        .await
        .map_err(|e| CliError::Transport(format!("Hyperlane delivery query failed: {e}")))?
        .into_inner();

    let delivery_str = if response.delivered { "Delivered" } else { "Pending (not yet delivered)" };

    match args.protocol {
        BridgeProtocol::WarpRoute => {
            dispatcher.output.success(format!(
                "Bridge status ({chain_label})\n\
                 MessageID: 0x{message_id}\n\
                 Status: {delivery_str}",
            ));
        }
        BridgeProtocol::Cctp => {
            use std::fmt::Write;
            let mut cctp_status = format!(
                "CCTP Bridge status ({chain_label})\n\
                 MessageID: 0x{message_id}\n\
                 Hyperlane Delivery: {delivery_str}",
            );

            if let Some(handler) = &args.cctp_handler {
                let cctp_info = query_cctp_status(dispatcher, handler, message_id).await;
                let _ = write!(cctp_status, "\n Attestation: {cctp_info}");
            } else {
                cctp_status.push_str(
                    "\n CCTP Fulfillment: (pass --cctp-handler to query)",
                );
            }

            dispatcher.output.success(cctp_status);
        }
    }

    Ok(())
}

#[derive(Clone, prost::Message)]
struct CctpQReq {
    #[prost(string, tag = "1")]
    address: String,
    #[prost(bytes = "vec", tag = "2")]
    query_data: Vec<u8>,
}

#[derive(Clone, prost::Message)]
struct CctpQResp {
    #[prost(bytes = "vec", tag = "1")]
    data: Vec<u8>,
}

#[derive(serde::Deserialize)]
struct CctpPendingResp {
    transfer: Option<serde_json::Value>,
}

async fn query_cctp_status(
    dispatcher: &Dispatcher,
    handler: &str,
    message_hash: &str,
) -> String {
    let query_msg = serde_json::json!({
        "pending_by_hash": { "hash": message_hash }
    });
    let query_data = match serde_json::to_vec(&query_msg) {
        Ok(d) => d,
        Err(e) => return format!("serialization error: {e}"),
    };

    let channel = match crate::transport::connect(&dispatcher.config.rpc_url).await {
        Ok(c) => c,
        Err(e) => return format!("connection error: {e}"),
    };

    let mut grpc = tonic::client::Grpc::new(channel);
    if let Err(e) = grpc.ready().await {
        return format!("service not ready: {e}");
    }

    let path = match "/cosmwasm.wasm.v1.Query/SmartContractState"
        .parse::<http::uri::PathAndQuery>()
    {
        Ok(p) => p,
        Err(e) => return format!("path error: {e}"),
    };

    let codec: tonic_prost::ProstCodec<CctpQReq, CctpQResp> =
        tonic_prost::ProstCodec::default();

    let resp = match grpc
        .unary(
            tonic::Request::new(CctpQReq {
                address: handler.to_string(),
                query_data,
            }),
            path,
            codec,
        )
        .await
    {
        Ok(r) => r.into_inner(),
        Err(e) => return format!("query failed: {e}"),
    };

    match serde_json::from_slice::<CctpPendingResp>(&resp.data) {
        Ok(pr) => match pr.transfer {
            Some(_) => "Pending (awaiting attestation/fulfillment)".to_string(),
            None => "Fulfilled (transfer completed, USDC minted)".to_string(),
        },
        Err(e) => format!("response parse error: {e}"),
    }
}
