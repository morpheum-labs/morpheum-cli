#![allow(unused_assignments)]

mod cli;
mod config;
mod dispatcher;
mod error;
mod keyring;
mod output;
mod status;
mod tx;
mod query;
#[cfg(feature = "_transport")]
mod transport;
mod utils;
#[allow(dead_code, clippy::all, clippy::pedantic)]
mod xchain;
mod mwvm;
mod mcp;
mod a2a;
mod keys;

use clap::Parser;
use miette::Result as MietteResult;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::cli::Cli;
use crate::config::MorpheumConfig;
use crate::dispatcher::Dispatcher;
use crate::keyring::KeyringManager;
use crate::output::Output;

#[tokio::main]
async fn main() -> MietteResult<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    miette::set_panic_hook();

    let cli = Cli::parse();

    let mut config = MorpheumConfig::load()?;

    if let Some(chain_id) = &cli.global.chain_id {
        config.chain_id = chain_id.clone();
    }
    if let Some(rpc) = &cli.global.rpc {
        config.rpc_url = rpc.clone();
    }

    let output = Output::new(cli.global.output);
    let keyring = KeyringManager::new(config.clone());
    let dispatcher = Dispatcher::new(config, keyring, output);

    dispatcher.execute(cli.command).await?;

    Ok(())
}
