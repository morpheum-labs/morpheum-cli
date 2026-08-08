//! Shared utilities for Morpheum CLI transaction and query handlers.

#[cfg(feature = "_tx")]
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

        // Subtract a 2-second safety margin so the server always sees this
        // timestamp as "in the past". The chain validates
        // `now_truncated.wrapping_sub(ts_ms) <= window_ms` using u32
        // arithmetic — even 1ms of clock skew in the wrong direction causes
        // wrapping_sub to overflow to near u32::MAX, triggering rejection.
        // 2s is negligible against the 500s window but prevents all edge cases.
        #[allow(clippy::cast_possible_truncation)]
        let ts_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| (d.as_millis() as u32).wrapping_sub(2000));

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

        // Phase M3 — bind the signature to this chain instance so it cannot be
        // replayed onto another chain sharing our `chain_id`. Sourced from
        // operator configuration, never from `rpc_url`: see `GenesisHash`.
        //
        // Left unbound when unconfigured, which validators still accept while
        // the strict genesis fork is advisory. The warning is deliberate — an
        // unbound signature is replayable, and that should not be silent.
        let mut builder = morpheum_signing_native::native(signer)
            .chain_id(&dispatcher.config.chain_id)
            .memo(memo.unwrap_or_default())
            .with_nonce(nonce)
            .add_message(message);
        match dispatcher.config.genesis_hash {
            Some(genesis_hash) => {
                builder = builder.with_genesis_hash(*genesis_hash.as_bytes());
            }
            None => dispatcher.output.warn(
                "genesis_hash is not configured — this signature is not bound to a chain \
                 instance and is replayable on any chain sharing this chain_id. Set it with \
                 `morpheum config set genesis_hash <hex>`.",
            ),
        }
        let signed_tx = builder.sign().await.map_err(CliError::Signing)?;

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

        // Routed-shard surface (one site for every command). A txhash is an
        // admission receipt, not finality — the shard lets the user correlate
        // with per-shard status/health surfaces while they reconcile via
        // `tx.v1.Query/QueryTxStatus`.
        if let Some(shard_id) = response.shard_id {
            dispatcher
                .output
                .info(format!("Routed to shard {shard_id}"));
        }

        Ok(response.txhash)
    }
}

#[cfg(feature = "_tx")]
pub use broadcast::sign_and_broadcast;
