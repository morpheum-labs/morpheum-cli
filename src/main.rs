#![allow(unused_assignments)]

mod a2a;
mod account;
mod cli;
mod config;
mod dispatcher;
mod error;
mod keyring;
mod keys;
mod mcp;
mod mwvm;
mod output;
mod query;
mod status;
// Dialled only by the broadcast path; a query-only build has no
// caller for it.
#[cfg(feature = "_tx")]
mod transport;
mod tx;
mod utils;
#[allow(dead_code, clippy::all, clippy::pedantic)]
mod xchain;

use clap::Parser;
use miette::Result as MietteResult;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::cli::Cli;
use crate::config::MorpheumConfig;
use crate::dispatcher::Dispatcher;

#[tokio::main]
async fn main() -> MietteResult<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    miette::set_panic_hook();

    let cli = Cli::parse();

    let config = MorpheumConfig::load()?;
    let dispatcher = Dispatcher::new(config, cli.global);

    dispatcher.execute(cli.command).await?;

    Ok(())
}
