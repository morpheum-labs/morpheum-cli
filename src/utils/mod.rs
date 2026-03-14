//! Shared utilities for Morpheum CLI transaction and query handlers.

#[cfg(feature = "_transport")]
mod broadcast {
    use morpheum_signing_native::NativeSigner;

    use crate::dispatcher::Dispatcher;
    use crate::error::CliError;

    /// Resolves the on-chain nonce for the given address, increments the
    /// monotonic counter, and attaches the current wall-clock timestamp.
    async fn resolve_nonce(
        channel: &tonic::transport::Channel,
        address: &str,
    ) -> Result<morpheum_proto::tx::v1::Nonce, CliError> {
        let mut auth_client =
            morpheum_proto::auth::v1::query_client::QueryClient::new(channel.clone());

        let resp = auth_client
            .query_nonce_state(morpheum_proto::auth::v1::QueryNonceStateRequest {
                address: address.to_string(),
            })
            .await
            .map_err(|e| CliError::Transport(format!("nonce query failed: {e}")))?
            .into_inner();

        let last_monotonic = resp.state.as_ref().map_or(0, |s| s.last_monotonic);

        #[allow(clippy::cast_possible_truncation)]
        let ts_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u32)
            .unwrap_or(0);

        Ok(morpheum_proto::tx::v1::Nonce {
            monotonic: last_monotonic.wrapping_add(1),
            ts_ms,
            sub: 0,
        })
    }

    /// Signs a single-message transaction and broadcasts it via `IngressService/SubmitTx`.
    pub async fn sign_and_broadcast(
        signer: NativeSigner,
        dispatcher: &Dispatcher,
        message: morpheum_signing_native::Any,
        memo: Option<String>,
    ) -> Result<String, CliError> {
        use morpheum_signing_native::signer::Signer;

        let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
        let address = hex::encode(signer.account_id().0);
        let nonce = resolve_nonce(&channel, &address).await?;

        let signed_tx = morpheum_signing_native::native(signer)
            .chain_id(&dispatcher.config.chain_id)
            .memo(memo.unwrap_or_default())
            .with_nonce(nonce)
            .add_message(message)
            .sign()
            .await
            .map_err(CliError::Signing)?;

        let req = morpheum_proto::tx::v1::SubmitTxRequest {
            tx: Some(signed_tx.tx().clone()),
            ..Default::default()
        };

        let mut client =
            morpheum_proto::tx::v1::ingress_service_client::IngressServiceClient::new(channel);

        let response = client
            .submit_tx(tonic::Request::new(req))
            .await
            .map_err(|e| CliError::Transport(format!("SubmitTx failed: {e}")))?
            .into_inner();

        if !response.accepted {
            return Err(CliError::Transport(format!(
                "transaction rejected: {}",
                response.error_message
            )));
        }

        Ok(response.txhash)
    }
}

#[cfg(feature = "_transport")]
pub use broadcast::sign_and_broadcast;
