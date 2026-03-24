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

    /// Returns an `SdkConfig` derived from the CLI's current configuration.
    pub fn sdk_config(&self) -> morpheum_sdk_core::SdkConfig {
        morpheum_sdk_core::SdkConfig::new(
            self.config.rpc_url.clone(),
            self.config.chain_id.clone(),
        )
    }

    /// Creates a `GrpcTransport` connected to the configured RPC endpoint.
    pub async fn grpc_transport(&self) -> Result<morpheum_sdk_native::GrpcTransport, CliError> {
        morpheum_sdk_native::GrpcTransport::connect(&self.config.rpc_url)
            .await
            .map_err(CliError::Sdk)
    }

    /// Creates a `BankClient` backed by a live gRPC connection.
    #[cfg(feature = "bank")]
    pub async fn bank_client(&self) -> Result<morpheum_sdk_native::bank::BankClient, CliError> {
        let transport = self.grpc_transport().await?;
        Ok(morpheum_sdk_native::bank::BankClient::new(
            self.sdk_config(),
            Box::new(transport),
        ))
    }

    /// Creates an `IdentityClient` backed by a live gRPC connection.
    #[cfg(feature = "identity")]
    pub async fn identity_client(
        &self,
    ) -> Result<morpheum_sdk_native::identity::IdentityClient, CliError> {
        let transport = self.grpc_transport().await?;
        Ok(morpheum_sdk_native::identity::IdentityClient::new(
            self.sdk_config(),
            Box::new(transport),
        ))
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
