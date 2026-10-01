use clap::{Args, Subcommand};

use morpheum_sdk_native::identity::{
    AgentMetadataCardInput, Capability, RegisterAgentBuilder, RegisterAgentRequest,
    RegistrationOwner,
};

use crate::dispatcher::Dispatcher;
use crate::error::CliError;
use crate::utils::sign_and_broadcast;

/// Transaction commands for the `identity` module.
#[derive(Subcommand)]
pub enum IdentityCommands {
    /// Register a new agent identity on-chain
    Register(RegisterArgs),
}

#[derive(Args)]
pub struct RegisterArgs {
    /// DID for the new agent (e.g. did:agent:alpha-trader-v3)
    #[arg(long)]
    pub did: String,

    /// Display name for the agent
    #[arg(long)]
    pub display_name: String,

    /// Short description of the agent
    #[arg(long)]
    pub description: Option<String>,

    /// Comma-separated capabilities (trade, evaluate, delegate, analyze)
    #[arg(long, value_delimiter = ',')]
    pub capabilities: Vec<String>,

    /// Key name to sign with (from `morpheum keys list`)
    #[arg(long, default_value = "default")]
    pub from: String,

    /// Mark as self-owned (autonomous) agent
    #[arg(long, conflicts_with = "owner_agent_hash")]
    pub self_owned: bool,

    /// Owner agent of the new agent, named by its agent hash hex(SHA256(DID)).
    /// Omitted, the signer's bound agent owns it. An account address is not an
    /// agent hash.
    #[arg(long)]
    pub owner_agent_hash: Option<String>,

    /// Optional memo for the transaction
    #[arg(long)]
    pub memo: Option<String>,
}

pub async fn execute(cmd: IdentityCommands, dispatcher: Dispatcher) -> Result<(), CliError> {
    match cmd {
        IdentityCommands::Register(args) => register(args, dispatcher).await,
    }
}

async fn register(args: RegisterArgs, dispatcher: Dispatcher) -> Result<(), CliError> {
    let signer = dispatcher.keyring.get_native_signer(&args.from)?;
    let request = register_agent_request(&args)?;

    let txhash = sign_and_broadcast(signer, &dispatcher, request.to_any(), args.memo).await?;

    dispatcher.output.success(format!(
        "Agent registered!\nDID: {}\nTxHash: {txhash}",
        args.did
    ));

    Ok(())
}

/// The registration `args` describe. The owner comes from the flags alone:
/// the signing key's account is never an agent hash.
fn register_agent_request(args: &RegisterArgs) -> Result<RegisterAgentRequest, CliError> {
    let owner = match (&args.owner_agent_hash, args.self_owned) {
        (_, true) => RegistrationOwner::SelfOwned,
        (Some(hash), false) => RegistrationOwner::Agent(hash.clone()),
        (None, false) => RegistrationOwner::Signer,
    };

    let caps = capabilities_to_bitflags(&args.capabilities);
    let metadata = AgentMetadataCardInput {
        display_name: args.display_name.clone(),
        description: args.description.clone().unwrap_or_default(),
        tags: String::new(),
        version: "1.0.0".into(),
        capabilities: caps,
    };

    RegisterAgentBuilder::new()
        .did(&args.did)
        .owner(owner)
        .metadata(metadata)
        .capabilities(caps)
        .build()
        .map_err(CliError::Sdk)
}

fn capabilities_to_bitflags(caps: &[String]) -> u64 {
    let mut flags = 0u64;
    for cap in caps {
        flags |= match cap.to_lowercase().as_str() {
            "trade" => Capability::TRADE,
            "evaluate" => Capability::EVALUATE,
            "manage" => Capability::MANAGE,
            "memory" => Capability::MEMORY,
            _ => 0,
        };
    }
    flags
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[derive(Parser)]
    struct Cmd {
        #[command(flatten)]
        args: RegisterArgs,
    }

    fn parse(extra: &[&str]) -> Result<RegisterArgs, clap::Error> {
        let base = [
            "register",
            "--did",
            "did:agent:test",
            "--display-name",
            "Test",
        ];
        Cmd::try_parse_from(base.iter().chain(extra).copied()).map(|cmd| cmd.args)
    }

    #[test]
    fn identity_register_never_names_the_account_as_owner() {
        let args = parse(&[]).expect("no owner flag parses");
        let request = register_agent_request(&args).expect("request builds");
        assert_eq!(request.owner, RegistrationOwner::Signer);

        let args = parse(&["--self-owned"]).expect("--self-owned parses");
        let request = register_agent_request(&args).expect("request builds");
        assert_eq!(request.owner, RegistrationOwner::SelfOwned);
    }

    #[test]
    fn identity_register_names_the_given_owner() {
        let owner = "ab".repeat(32);
        let args = parse(&["--owner-agent-hash", &owner]).expect("owner flag parses");
        let request = register_agent_request(&args).expect("request builds");
        assert_eq!(request.owner, RegistrationOwner::Agent(owner));
    }

    #[test]
    fn identity_register_refuses_self_owned_with_an_owner() {
        let owner = "ab".repeat(32);
        let Err(err) = parse(&["--self-owned", "--owner-agent-hash", &owner]) else {
            panic!("--self-owned with --owner-agent-hash must not parse");
        };
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
