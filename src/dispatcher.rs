use crate::cli::{Commands, GlobalArgs};
use crate::config::MorpheumConfig;
use crate::error::CliError;
use crate::keyring::KeyringManager;
use crate::output::Output;
#[cfg(feature = "_tx")]
use crate::utils::TxMessage;
#[cfg(feature = "_tx")]
use morpheum_signing_native::{builder::TxBuilder, NativeSigner, TxGasLimit};

/// Central dispatcher for the entire Morpheum CLI.
///
/// Holds the shared context (config, keyring, output, and the gas limit
/// transactions declare) that every command needs.
/// Adding a new top-level command group requires only one match arm here and
/// the corresponding module implementation.
#[derive(Debug)]
pub struct Dispatcher {
    pub config: MorpheumConfig,
    pub keyring: KeyringManager,
    pub output: Output,
    /// `--gas-limit`; `None` leaves the declaration to the message (see
    /// [`TxMessage::declared_gas_limit`]). Private and read only by
    /// [`Self::tx_builder`], the one way it reaches a transaction.
    #[cfg(feature = "_tx")]
    gas_limit: Option<TxGasLimit>,
}

impl Dispatcher {
    /// Builds the context for one invocation: the loaded `config` with the
    /// global flags applied over it.
    pub fn new(mut config: MorpheumConfig, global: GlobalArgs) -> Self {
        if let Some(chain_id) = global.chain_id {
            config.chain_id = chain_id;
        }
        if let Some(rpc) = global.rpc {
            config.rpc_url = rpc;
        }
        Self {
            keyring: KeyringManager::new(config.clone()),
            output: Output::new(global.output),
            #[cfg(feature = "_tx")]
            gas_limit: global.gas_limit,
            config,
        }
    }

    /// Starts the transaction that carries `message`, signed by `signer`,
    /// declaring the gas limit [`TxMessage::declared_gas_limit`] decides
    /// from `--gas-limit`.
    ///
    /// Every transaction the CLI signs starts here, through
    /// `utils::sign_and_broadcast`, so the flag and a message's own limit
    /// apply to all of them; `clippy.toml` refuses every other way to start
    /// one.
    ///
    /// # Errors
    ///
    /// As [`TxMessage::declared_gas_limit`].
    #[cfg(feature = "_tx")]
    pub fn tx_builder(
        &self,
        signer: NativeSigner,
        message: TxMessage,
    ) -> Result<TxBuilder<NativeSigner>, CliError> {
        let gas_limit = message.declared_gas_limit(self.gas_limit)?;
        #[allow(clippy::disallowed_methods)] // the one place a transaction starts
        let builder = morpheum_signing_native::native(signer);
        Ok(builder.gas_limit(gas_limit).add_message(message.into_any()))
    }

    /// Returns an `SdkConfig` derived from the CLI's current configuration.
    #[cfg(feature = "_transport")]
    pub fn sdk_config(&self) -> morpheum_sdk_core::SdkConfig {
        morpheum_sdk_core::SdkConfig::new(self.config.rpc_url.clone(), self.config.chain_id.clone())
    }

    /// Creates a `GrpcTransport` connected to the configured RPC endpoint.
    #[cfg(feature = "_transport")]
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

    /// Routes the parsed command to the appropriate module.
    pub async fn execute(self, cmd: Commands) -> Result<(), CliError> {
        match cmd {
            Commands::Tx(sub) => crate::tx::execute(sub, self).await,
            Commands::Query(sub) => crate::query::execute(sub, self).await,
            Commands::Mwvm(sub) => crate::mwvm::execute(sub, self).await,
            Commands::Mcp(sub) => crate::mcp::execute(sub, self).await,
            Commands::A2a(sub) => crate::a2a::execute(sub, self).await,
            Commands::Keys(sub) => crate::key_management::execute(sub, self).await,
            Commands::Status => crate::status::execute(self).await,
            Commands::Config(sub) => crate::config::execute(sub, self).await,
        }
    }
}
