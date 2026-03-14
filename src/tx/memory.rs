use clap::{Args, Subcommand};

use morpheum_signing_native::signer::Signer;
use morpheum_sdk_native::memory::{
    StoreEntryBuilder, UpdateEntryBuilder, DeleteEntryBuilder, MemoryEntryType,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Transaction commands for the `memory` module.
///
/// Covers persistent memory CRUD operations — store, update, and delete
/// entries in an agent's on-chain memory namespace.
#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Store a new memory entry for an agent
    Store(StoreArgs),

    /// Update an existing memory entry
    Update(UpdateArgs),

    /// Delete a memory entry
    Delete(DeleteArgs),
}

#[derive(Args)]
pub struct StoreArgs {
    /// Agent hash (owner of the memory namespace)
    #[arg(long)]
    pub agent_hash: String,

    /// Entry key (unique within the agent's namespace)
    #[arg(long)]
    pub key: String,

    /// Value as a UTF-8 string (will be stored as bytes)
    #[arg(long)]
    pub value: String,

    /// Entry type (episodic, semantic, procedural, vector, custom)
    #[arg(long, value_parser = parse_entry_type)]
    pub entry_type: MemoryEntryType,

    /// Expiry timestamp in seconds since epoch (0 = never)
    #[arg(long, default_value = "0")]
    pub expires_at: u64,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// Entry key to update
    #[arg(long)]
    pub key: String,

    /// New value as a UTF-8 string
    #[arg(long)]
    pub value: String,

    /// New expiry timestamp (0 = never)
    #[arg(long)]
    pub expires_at: Option<u64>,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Agent hash
    #[arg(long)]
    pub agent_hash: String,

    /// Entry key to delete
    #[arg(long)]
    pub key: String,

    /// Key name to sign with
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Optional memo
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: MemoryCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        MemoryCommands::Store(args) => store(args, &dispatcher).await,
        MemoryCommands::Update(args) => update(args, &dispatcher).await,
        MemoryCommands::Delete(args) => delete(args, &dispatcher).await,
    }
}

async fn store(args: StoreArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let mut builder = StoreEntryBuilder::new()
        .agent_hash(&args.agent_hash)
        .key(&args.key)
        .value(args.value.as_bytes().to_vec())
        .entry_type(args.entry_type)
        .owner_signature(owner_sig);

    if args.expires_at > 0 {
        builder = builder.expires_at(args.expires_at);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Memory entry stored\nAgent: {}, Key: {}\nType: {}\nTxHash: {}",
        args.agent_hash, args.key, args.entry_type, txhash,
    ));

    Ok(())
}

async fn update(args: UpdateArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let mut builder = UpdateEntryBuilder::new()
        .agent_hash(&args.agent_hash)
        .key(&args.key)
        .new_value(args.value.as_bytes().to_vec())
        .owner_signature(owner_sig);

    if let Some(expires) = args.expires_at {
        builder = builder.new_expires_at(expires);
    }

    let request = builder.build().map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Memory entry updated\nAgent: {}, Key: {}\nTxHash: {}",
        args.agent_hash, args.key, txhash,
    ));

    Ok(())
}

async fn delete(args: DeleteArgs, dispatcher: &Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let owner_sig = signer.public_key().to_proto_bytes();

    let request = DeleteEntryBuilder::new()
        .agent_hash(&args.agent_hash)
        .key(&args.key)
        .owner_signature(owner_sig)
        .build()
        .map_err(CliError::Sdk)?;

    let txhash = crate::utils::sign_and_broadcast(
        signer, dispatcher, request.to_any(), args.memo,
    )
    .await?;

    dispatcher.output.success(format!(
        "Memory entry deleted\nAgent: {}, Key: {}\nTxHash: {}",
        args.agent_hash, args.key, txhash,
    ));

    Ok(())
}

fn parse_entry_type(s: &str) -> Result<MemoryEntryType, String> {
    match s.to_lowercase().as_str() {
        "episodic" => Ok(MemoryEntryType::Episodic),
        "semantic" => Ok(MemoryEntryType::Semantic),
        "procedural" => Ok(MemoryEntryType::Procedural),
        "vector" => Ok(MemoryEntryType::Vector),
        "custom" => Ok(MemoryEntryType::Custom),
        other => Err(format!(
            "unknown entry type '{other}'; expected: episodic, semantic, procedural, vector, custom"
        )),
    }
}
