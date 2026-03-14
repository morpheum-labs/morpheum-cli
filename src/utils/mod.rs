//! Shared utilities for Morpheum CLI transaction and query handlers.
//!
//! Re-exports extension traits and the canonical `sign_and_broadcast`
//! function. Gated behind the `modules` feature alongside `tx/` and `query/`.

#[cfg(feature = "modules")]
pub mod tx_builder_ext;
#[cfg(feature = "modules")]
pub mod query_client_ext;

#[cfg(feature = "modules")]
pub use tx_builder_ext::TxBuilderExt;
#[cfg(feature = "modules")]
pub use query_client_ext::QueryClientExt;

#[cfg(feature = "modules")]
mod broadcast {
    use morpheum_sdk_native::core::prelude::Any;
    use morpheum_sdk_native::MorpheumSdk;
    use morpheum_signing_native::builder::TxBuilder;
    use morpheum_signing_native::signer::Signer;

    use crate::dispatcher::Dispatcher;
    use crate::error::CliError;

    /// Signs a single-message transaction and broadcasts it to the network.
    ///
    /// Canonical sign-and-broadcast flow used by every `tx/` handler.
    pub async fn sign_and_broadcast<S: Signer + Send>(
        signer: S,
        dispatcher: &Dispatcher,
        message: Any,
        memo: Option<String>,
    ) -> Result<(), CliError> {
        let signed_tx = TxBuilder::new(signer)
            .chain_id(&dispatcher.config.chain_id)
            .memo(memo.unwrap_or_default())
            .add_message(message)
            .sign()
            .await?;

        let sdk = MorpheumSdk::new(&dispatcher.config.rpc_url, &dispatcher.config.chain_id);
        sdk.transport()
            .broadcast_tx(signed_tx.raw_bytes().to_vec())
            .await?;

        Ok(())
    }
}

#[cfg(feature = "modules")]
pub use broadcast::sign_and_broadcast;
