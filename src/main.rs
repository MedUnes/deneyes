use crate::errors::DneyesError;
use clap::Parser;
use cli::*;
use std::collections::HashMap;
use serde::Deserialize;

pub mod cli;
pub mod commands;
pub mod dns;
pub mod errors;
pub mod http;
pub mod utils;
#[derive(Debug,Deserialize)]
struct DNeyeSDnsConfig {
    pub server_list_url: String,
    pub country_list: HashMap<String, String>,
}
#[derive(Debug,Deserialize)]
struct DNeyeSConfig {
    dns: DNeyeSDnsConfig,
}
#[tokio::main]
async fn main() -> Result<(), DneyesError> {
    let mode = Cli::parse().mode;
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()
        .unwrap();
   let cfg =  settings
            .try_deserialize::<DNeyeSConfig>()
            .expect("failed to read DNeyeSConfig from config");
    println!("{:#?}", cfg.dns.country_list.get("de").expect("Unknown country"));

    println!("Starting with mode: {:?}\n", mode);
    println!("Starting DNeyeS in {:?} mode", mode);
    match mode {
        Mode::Http => {
            commands::http::run().await?;
        }
        Mode::Dns => {
            commands::dns::run().await?;
        }
    }
    Ok(())
}
