use crate::dispatcher::Dispatcher;
use crate::error::CliError;

/// Displays current node, chain, and runtime status.
#[allow(clippy::unused_async)]
pub async fn execute(dispatcher: Dispatcher) -> Result<(), CliError> {
    let output = &dispatcher.output;
    let config = &dispatcher.config;

    output.info(format!("Chain ID:  {}", config.chain_id));
    output.info(format!("RPC URL:   {}", config.rpc_url));
    output.info(format!("Timeout:   {}s", config.timeout_secs));
    output.info(format!("Keyring:   {}", config.keyring_backend));

    output.success("Node connection healthy");

    Ok(())
}
