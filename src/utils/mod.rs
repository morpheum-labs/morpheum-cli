//! Shared utilities for Morpheum CLI transaction and query handlers.

#[cfg(feature = "_tx")]
mod broadcast {
    use morpheum_signing_native::builder::TxBuilder;
    use morpheum_signing_native::NativeSigner;

    use super::TxMessage;
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
        message: impl Into<TxMessage>,
        memo: Option<String>,
    ) -> Result<String, CliError> {
        use morpheum_signing_native::signer::Signer;

        let channel = crate::transport::connect(&dispatcher.config.rpc_url).await?;
        let address = hex::encode(signer.account_id().0);
        let nonce = resolve_nonce(&channel, &address).await?;

        let signed_tx = unsigned_tx(signer, dispatcher, message.into(), memo, nonce)?
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

    /// The transaction [`sign_and_broadcast`] signs, built without network
    /// I/O: started by `Dispatcher::tx_builder` (which declares its gas
    /// limit), then bound to the configured chain.
    fn unsigned_tx(
        signer: NativeSigner,
        dispatcher: &Dispatcher,
        message: TxMessage,
        memo: Option<String>,
        nonce: morpheum_proto::tx::v1::Nonce,
    ) -> Result<TxBuilder<NativeSigner>, CliError> {
        // Bind the signature to this chain instance so it cannot be replayed
        // onto another chain sharing our `chain_id`. Sourced from operator
        // configuration, never from `rpc_url`: see `GenesisHash`.
        //
        // When unconfigured, warn with the command that fixes it; `sign()`
        // refuses to build a preimage that binds no chain.
        let mut builder = dispatcher
            .tx_builder(signer, message)?
            .chain_id(&dispatcher.config.chain_id)
            .memo(memo.unwrap_or_default())
            .with_nonce(nonce);
        match dispatcher.config.genesis_hash {
            Some(genesis_hash) => {
                builder = builder.with_genesis_hash(*genesis_hash.as_bytes());
            }
            None => dispatcher.output.warn(
                "genesis_hash is not configured, so signing will be refused. Set it from \
                 operator configuration (the chain's published genesis hash, not the node \
                 you submit to) with `morpheum config set genesis_hash <hex>`.",
            ),
        }
        Ok(builder)
    }

    #[cfg(test)]
    mod tests {
        use clap::Parser;
        use morpheum_signing_native::{Any, TxGasLimit, DEFAULT_GAS_LIMIT};

        use super::*;
        use crate::cli::Cli;
        use crate::config::MorpheumConfig;

        fn noop() -> Any {
            Any {
                type_url: "/morpheum.test.v1.MsgNoop".to_string(),
                value: Vec::new(),
            }
        }

        /// Builds the dispatcher from `argv` the way `main` does (over a
        /// configuration bound to a genesis hash), signs `message` in the
        /// transaction `sign_and_broadcast` builds, and returns the gas limit
        /// that transaction declares, read back with the chain's own
        /// predicate.
        async fn declared_gas_limit(
            argv: &[&str],
            message: TxMessage,
        ) -> Result<TxGasLimit, CliError> {
            let cli = Cli::try_parse_from(argv).expect("argv parses");
            let config = MorpheumConfig {
                genesis_hash: Some("5a".repeat(32).parse().expect("32 bytes of hex")),
                ..MorpheumConfig::default()
            };
            let dispatcher = Dispatcher::new(config, cli.global);
            let nonce = morpheum_proto::tx::v1::Nonce {
                monotonic: 1,
                ts_ms: 0,
                sub: 0,
            };
            let signed = unsigned_tx(
                NativeSigner::from_seed(&[7; 32]),
                &dispatcher,
                message,
                None,
                nonce,
            )?
            .sign()
            .await
            .expect("a one-message transaction bound to a genesis hash signs");
            Ok(TxGasLimit::declared_by(signed.tx())
                .expect("the signed transaction declares a gas limit"))
        }

        /// The transaction `sign_and_broadcast` signs declares `--gas-limit`
        /// when it is given and the signing SDK's default otherwise, and
        /// never less than the gas limit its message states for itself:
        /// unset, the declaration rises to that limit; set below it, nothing
        /// is signed. The flagged value differs from the default, so the
        /// first assertion cannot pass by the flag being dropped.
        #[tokio::test]
        async fn the_signed_transaction_declares_the_flag_or_at_least_the_message_limit() {
            let flagged = TxGasLimit::new(2_000_000).expect("in range");
            assert_ne!(flagged, DEFAULT_GAS_LIMIT);
            let own = TxGasLimit::new(DEFAULT_GAS_LIMIT.get() * 4).expect("in range");

            let status = ["morpheum", "status"];
            let flag = ["morpheum", "status", "--gas-limit", "2000000"];
            let low_flag = ["morpheum", "status", "--gas-limit", "1"];
            assert_eq!(
                declared_gas_limit(&flag, noop().into())
                    .await
                    .expect("signs"),
                flagged,
            );
            assert_eq!(
                declared_gas_limit(&status, noop().into())
                    .await
                    .expect("signs"),
                DEFAULT_GAS_LIMIT,
            );
            let vm = || TxMessage::with_own_gas_limit(noop(), own);
            assert_eq!(declared_gas_limit(&status, vm()).await.expect("signs"), own);
            let refusal = declared_gas_limit(&low_flag, vm())
                .await
                .expect_err("a flag below the message's own limit signs nothing");
            assert!(
                matches!(refusal, CliError::InvalidInput { .. }),
                "{refusal}"
            );
        }
    }
}

#[cfg(feature = "_tx")]
mod tx_message;

#[cfg(feature = "_tx")]
pub use broadcast::sign_and_broadcast;
#[cfg(feature = "_tx")]
pub use tx_message::TxMessage;
