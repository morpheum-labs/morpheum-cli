use miette::Diagnostic;
use thiserror::Error;

/// Unified error type for the Morpheum CLI.
///
/// Wraps external errors (SDK, signing, keyring, config) with user-friendly
/// diagnostic codes and help text. All module `execute` functions return
/// `Result<()>` which converts to `miette::Report` at the boundary in `main.rs`.
#[derive(Error, Diagnostic, Debug)]
pub enum CliError {
    #[error("SDK error: {0}")]
    #[diagnostic(code(morpheum::cli::sdk))]
    Sdk(#[from] morpheum_sdk_native::SdkError),

    #[error("Signing error: {0}")]
    #[diagnostic(code(morpheum::cli::signing))]
    Signing(#[from] morpheum_signing_native::error::SigningError),

    #[error("Failed to load or parse configuration")]
    #[diagnostic(
        code(morpheum::cli::config),
        help("Run `morpheum config show` to inspect or reset your configuration")
    )]
    Config(#[from] confy::ConfyError),

    #[error("Keyring operation failed: {0}")]
    #[diagnostic(code(morpheum::cli::keyring))]
    Keyring(#[from] keyring::Error),

    #[error("Agent not found: {id}")]
    #[diagnostic(
        code(morpheum::cli::agent_not_found),
        help("Verify the agent ID with `morpheum query agent-registry get`")
    )]
    #[allow(dead_code)]
    AgentNotFound { id: String },

    #[error("Invalid input: {reason}")]
    #[diagnostic(code(morpheum::cli::invalid_input))]
    InvalidInput { reason: String },

    #[error("{context}")]
    #[diagnostic(code(morpheum::cli::io))]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Transport error: {0}")]
    #[diagnostic(
        code(morpheum::cli::transport),
        help("Check --rpc endpoint and ensure the node is running")
    )]
    #[allow(dead_code)]
    Transport(String),

    #[error("EVM operation failed: {0}")]
    #[diagnostic(
        code(morpheum::cli::evm),
        help("Check RPC URL and contract addresses in your chain configuration")
    )]
    Evm(String),

    #[error("SVM (Solana) operation failed: {0}")]
    #[diagnostic(
        code(morpheum::cli::svm),
        help("Check Solana RPC URL, program IDs, and account balances")
    )]
    Svm(String),

    #[error("CosmWasm operation failed: {0}")]
    #[diagnostic(
        code(morpheum::cli::cosmwasm),
        help("Check contract address, code ID, and message format")
    )]
    CosmWasm(String),

    #[error("Internal error: {0}")]
    #[diagnostic(code(morpheum::cli::internal))]
    #[allow(dead_code)]
    Internal(String),
}

impl CliError {
    pub fn invalid_input(reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            reason: reason.into(),
        }
    }

    #[allow(dead_code)]
    pub fn agent_not_found(id: impl Into<String>) -> Self {
        Self::AgentNotFound { id: id.into() }
    }

    #[allow(dead_code)]
    pub fn internal(context: impl Into<String>) -> Self {
        Self::Internal(context.into())
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            context: e.to_string(),
            source: e,
        }
    }
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, CliError>;
