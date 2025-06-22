use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(
    version = "0.1.0",
    about = "DNeyeS: Your DNS Eyes",
    long_about = "A multi-thread asynchronous low-level DNS and HTTP site scanner built in Rust"
)]
pub struct Cli {
    #[arg(value_enum, default_value = "dns")]
    pub mode: Mode,
}
#[derive(Clone, Debug, ValueEnum)]
pub enum Mode {
    Http,
    Dns,
}