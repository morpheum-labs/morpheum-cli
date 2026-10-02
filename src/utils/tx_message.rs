//! [`TxMessage`]: the message `sign_and_broadcast` signs, and the gas limit a
//! transaction carrying it declares.

use morpheum_signing_native::{Any, TxGasLimit, DEFAULT_GAS_LIMIT};

use crate::error::CliError;

/// The one message of a transaction the CLI signs, with the gas limit the
/// message states for itself when it states one.
///
/// A VM message carries a limit of its own (an SVM program call states its
/// compute limit), and it cannot use more gas than its transaction declares,
/// so [`Self::declared_gas_limit`] never declares less than that limit. A
/// native-module message states none; it converts from its `Any`.
#[derive(Debug)]
pub struct TxMessage {
    any: Any,
    own_gas_limit: Option<TxGasLimit>,
}

impl TxMessage {
    /// A message that states `own_gas_limit` for itself. The limit is a
    /// [`TxGasLimit`], so one that no transaction could declare (`0`, or
    /// above the per-transaction budget) cannot be attached.
    ///
    /// Gated with its only callers, the SVM commands, like the rest of the
    /// signing path: a build without them has no message that states a limit.
    #[cfg(any(test, feature = "svm"))]
    #[must_use]
    pub fn with_own_gas_limit(any: Any, own_gas_limit: TxGasLimit) -> Self {
        Self {
            any,
            own_gas_limit: Some(own_gas_limit),
        }
    }

    /// The gas limit a transaction carrying this message declares, given
    /// `--gas-limit` as `flag`: the flag when it is given, otherwise the
    /// signing SDK's `DEFAULT_GAS_LIMIT`, raised to the message's own limit
    /// when that is larger. Both are valid declarations, so the larger one is
    /// too.
    ///
    /// # Errors
    ///
    /// `flag` is below the message's own limit: the message could not run
    /// under it, so nothing is signed.
    pub fn declared_gas_limit(&self, flag: Option<TxGasLimit>) -> Result<TxGasLimit, CliError> {
        match (flag, self.own_gas_limit) {
            (Some(flag), Some(own)) if flag < own => Err(CliError::invalid_input(format!(
                "--gas-limit {flag} is below the gas limit of {own} this message states \
                 for itself; a message cannot use more gas than its transaction \
                 declares, so declare at least {own}",
                flag = flag.get(),
                own = own.get(),
            ))),
            (Some(flag), _) => Ok(flag),
            (None, Some(own)) => Ok(own.max(DEFAULT_GAS_LIMIT)),
            (None, None) => Ok(DEFAULT_GAS_LIMIT),
        }
    }

    /// The message itself, for the transaction body.
    #[must_use]
    pub fn into_any(self) -> Any {
        self.any
    }
}

impl From<Any> for TxMessage {
    /// A native-module message, which states no gas limit of its own.
    fn from(any: Any) -> Self {
        Self {
            any,
            own_gas_limit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop() -> Any {
        Any {
            type_url: "/morpheum.test.v1.MsgNoop".to_string(),
            value: Vec::new(),
        }
    }

    fn gas(units: u64) -> TxGasLimit {
        TxGasLimit::new(units).expect("a valid declaration")
    }

    /// A native-module message states no limit of its own: `--gas-limit`
    /// is declared as given anywhere in range, and the SDK default without
    /// it.
    #[test]
    fn a_native_message_declares_the_flag_or_the_default() {
        let native = TxMessage::from(noop());
        assert_eq!(
            native
                .declared_gas_limit(None)
                .expect("unset never refuses"),
            DEFAULT_GAS_LIMIT,
        );
        for flag in [TxGasLimit::MIN, DEFAULT_GAS_LIMIT, TxGasLimit::MAX] {
            assert_eq!(
                native
                    .declared_gas_limit(Some(flag))
                    .expect("any flag in range"),
                flag,
            );
        }
    }

    /// A message that states its own limit is never signed under less.
    /// Without the flag the declaration is the default, or the message's
    /// limit when that is larger, up to the whole per-transaction budget; a
    /// flag at or above the message's limit is declared as given.
    #[test]
    fn a_message_with_its_own_limit_declares_at_least_that_limit() {
        let above_default = gas(DEFAULT_GAS_LIMIT.get() + 1);
        for (own, unset) in [
            (TxGasLimit::MIN, DEFAULT_GAS_LIMIT),
            (gas(DEFAULT_GAS_LIMIT.get() - 1), DEFAULT_GAS_LIMIT),
            (DEFAULT_GAS_LIMIT, DEFAULT_GAS_LIMIT),
            (above_default, above_default),
            (TxGasLimit::MAX, TxGasLimit::MAX),
        ] {
            let vm = TxMessage::with_own_gas_limit(noop(), own);
            assert_eq!(
                vm.declared_gas_limit(None).expect("unset never refuses"),
                unset
            );
            assert_eq!(vm.declared_gas_limit(Some(own)).expect("at its limit"), own);
            assert_eq!(
                vm.declared_gas_limit(Some(TxGasLimit::MAX))
                    .expect("above its limit"),
                TxGasLimit::MAX,
            );
        }
    }

    /// A flag below the message's own limit is refused rather than signed,
    /// and the refusal names both numbers so the user can correct it.
    #[test]
    fn a_flag_below_the_message_limit_is_refused() {
        let own = gas(DEFAULT_GAS_LIMIT.get() + 1);
        let refusal = TxMessage::with_own_gas_limit(noop(), own)
            .declared_gas_limit(Some(DEFAULT_GAS_LIMIT))
            .expect_err("a flag below the message's own limit");
        assert!(
            matches!(refusal, CliError::InvalidInput { .. }),
            "{refusal}"
        );
        let text = refusal.to_string();
        for named in [DEFAULT_GAS_LIMIT.get(), own.get()] {
            assert!(
                text.contains(&named.to_string()),
                "must name {named}: {text}"
            );
        }
    }
}
