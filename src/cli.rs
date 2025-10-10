use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Command line interface definition for DNeyeS.
#[derive(Parser)]
#[command(
    version = "0.1.0",
    about = "DNeyeS: Your DNS Eyes",
    long_about = "A multi-thread asynchronous low-level DNS and HTTP site scanner built in Rust"
)]
pub struct Cli {
    /// Path to the configuration file.
    #[arg(
        long,
        value_name = "FILE",
        env = "DNEYES_CONFIG",
        default_value = "config.yaml"
    )]
    pub config: PathBuf,
    /// Command to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported operating modes of DNeyeS.
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Run DNS resolution monitoring.
    Dns,
    /// Run HTTP uptime monitoring.
    Http,
    /// Start the REST API server.
    Api,
}
