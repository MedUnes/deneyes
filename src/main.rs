use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::AppConfig;
use crate::errors::DneyesError;
use crate::telemetry::clickhouse::ClickhouseContext;
use crate::telemetry::sink;

pub mod cli;
pub mod commands;
pub mod config;
pub mod dns;
pub mod errors;
pub mod http;
pub mod telemetry;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), DneyesError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let config = AppConfig::load(&cli.config)?;

    match cli.command {
        Command::Dns => {
            let sinks = sink::build_sinks(&config.output).await?;
            commands::dns::run(&config, sinks.dns).await?;
        }
        Command::Http => {
            let sinks = sink::build_sinks(&config.output).await?;
            commands::http::run(sinks.http).await?;
        }
        Command::Api => {
            let clickhouse_cfg = config.output.clickhouse.clone().ok_or_else(|| {
                DneyesError::Config(
                    "ClickHouse configuration is required to run the API server".to_string(),
                )
            })?;
            let context = ClickhouseContext::new(&clickhouse_cfg).await?;
            commands::api::run(&config, context).await?;
        }
    }

    Ok(())
}
