//! gRPC transport for the Morpheum CLI.
//!
//! Provides direct access to Mormcore gRPC services for transaction
//! broadcasting and module queries.

use tonic::transport::Channel;

use crate::error::CliError;

/// Connects to a Mormcore gRPC endpoint and returns a `tonic::Channel`.
pub async fn connect(endpoint: &str) -> Result<Channel, CliError> {
    Channel::from_shared(endpoint.to_string())
        .map_err(|e| CliError::Transport(format!("invalid endpoint: {e}")))?
        .connect()
        .await
        .map_err(|e| CliError::Transport(format!("gRPC connect to {endpoint} failed: {e}")))
}
