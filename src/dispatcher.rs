use crate::cli::Commands;
use crate::config::MorpheumConfig;
use crate::error::CliError;
use crate::keyring::KeyringManager;
use crate::output::Output;

/// Central dispatcher for the entire Morpheum CLI.
///
/// Holds the shared context (config, keyring, output) that every command needs.
/// Adding a new top-level command group requires only one match arm here and
/// the corresponding module implementation.
#[derive(Debug)]
pub struct Dispatcher {
    pub config: MorpheumConfig,
    pub keyring: KeyringManager,
    pub output: Output,
}

impl Dispatcher {
    pub fn new(config: MorpheumConfig, keyring: KeyringManager, output: Output) -> Self {
        Self { config, keyring, output }
    }

    /// Routes the parsed command to the appropriate module.
    pub async fn execute(self, cmd: Commands) -> Result<(), CliError> {
        match cmd {
            Commands::Tx(sub) => crate::tx::execute(sub, self).await,
            Commands::Query(sub) => crate::query::execute(sub, self).await,
            Commands::Mwvm(sub) => crate::mwvm::execute(sub, self).await,
            Commands::Mcp(sub) => crate::mcp::execute(sub, self).await,
            Commands::A2a(sub) => crate::a2a::execute(sub, self).await,
            Commands::Keys(sub) => crate::keys::execute(sub, self).await,
            Commands::Status => crate::status::execute(self).await,
            Commands::Config(sub) => crate::config::execute(sub, self).await,
        }
    }
}
