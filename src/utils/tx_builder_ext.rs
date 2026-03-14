//! `tx_builder_ext.rs`
//!
//! Extension trait for `TxBuilder` from `morpheum-signing-native`.
//!
//! This is the **canonical** way every transaction module (`tx/*.rs`) builds
//! transactions in the CLI. It guarantees:
//! - Correct protobuf `Any` packing with full type URLs
//! - Fluent builder pattern preserved
//! - Zero boilerplate in individual tx files
//! - Built-in support for agent delegation via `TradingKeyClaim`
//!
//! Pure extension trait pattern — zero-cost, production-grade, and fully DRY.

use morpheum_signing_native::{Signer, TradingKeyClaim, TxBuilder};
use prost::Message;

/// Extension trait for `TxBuilder<S>` that provides clean, consistent, and DRY
/// methods for building transactions across all `tx/` modules in the CLI.
///
/// This is the single source of truth for transaction construction.
/// All `tx/` modules should import and use this trait.
pub trait TxBuilderExt<S: Signer>: Sized {
    /// Adds a protobuf message by automatically wrapping it in `prost::Any`
    /// using the standard `type.googleapis.com/...` type URL.
    fn add_proto_msg<M: Message>(self, msg: M) -> Self;

    /// Sets the chain ID for the transaction (fluent).
    fn with_chain_id(self, chain_id: impl Into<String>) -> Self;

    /// Sets an optional memo on the transaction (fluent).
    fn with_memo(self, memo: impl Into<String>) -> Self;

    /// Attaches a `TradingKeyClaim` for agent delegation flows.
    ///
    /// Essential for all agent-native commands (Pillar 2 thin adapters and Pillar 3 trust layer).
    fn with_trading_key_claim(self, claim: TradingKeyClaim) -> Self;
}

impl<S: Signer> TxBuilderExt<S> for TxBuilder<S> {
    fn add_proto_msg<M: Message>(self, msg: M) -> Self {
        let type_url = format!("type.googleapis.com/{}", M::NAME);
        self.add_typed_message(type_url, &msg)
    }

    fn with_chain_id(self, chain_id: impl Into<String>) -> Self {
        self.chain_id(chain_id)
    }

    fn with_memo(self, memo: impl Into<String>) -> Self {
        self.memo(memo)
    }

    fn with_trading_key_claim(self, claim: TradingKeyClaim) -> Self {
        self.with_trading_key_claim(claim)
    }
}