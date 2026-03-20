//! Bridge commands — cross-chain deposit, withdraw, and status.
//!
//! VM-generic dispatch: the `--chain` flag selects EVM or SVM, and the
//! appropriate SDK crate handles the heavy logic.

mod commands;

pub use commands::{BridgeCommands, execute};
