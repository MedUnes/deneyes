use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::errors::DneyesError;

/// Application wide configuration that covers DNS lookups, output sinks and the
/// public API layer.
#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    /// DNS specific configuration block.
    pub dns: DnsConfig,
    /// Output configuration describing where measurement results should be persisted.
    pub output: OutputConfig,
    /// Public API configuration used for the REST layer.
    pub api: ApiConfig,
}

impl AppConfig {
    /// Load configuration from the provided file system path using the `config`
    /// crate so we can support YAML, JSON or TOML without additional effort.
    pub fn load(path: &Path) -> Result<Self, DneyesError> {
        let builder = config::Config::builder().add_source(config::File::from(path));
        let cfg = builder.build()?;
        let config: AppConfig = cfg.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), DneyesError> {
        if self.dns.default_countries.is_empty() {
            return Err(DneyesError::Config(
                "`dns.default_countries` must contain at least one entry".to_string(),
            ));
        }

        match self.output.mode {
            OutputMode::File => {
                if self.output.file.is_none() {
                    return Err(DneyesError::Config(
                        "`output.file` block is required when mode is `file`".to_string(),
                    ));
                }
            }
            OutputMode::Clickhouse => {
                if self.output.clickhouse.is_none() {
                    return Err(DneyesError::Config(
                        "`output.clickhouse` block is required when mode is `clickhouse`"
                            .to_string(),
                    ));
                }
            }
            OutputMode::Both => {
                if self.output.file.is_none() || self.output.clickhouse.is_none() {
                    return Err(DneyesError::Config(
                        "`output.file` and `output.clickhouse` blocks are required when mode is `both`"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// DNS module configuration controlling resolver list retrieval and the
/// runtime behaviour.
#[derive(Clone, Debug, Deserialize)]
pub struct DnsConfig {
    /// Public DNS list provider URL.
    pub server_list_url: String,
    /// Mapping of ISO country code to human readable name.
    pub country_list: HashMap<String, String>,
    /// Default list of ISO country codes to query for DNS servers.
    #[serde(default = "default_countries")]
    pub default_countries: Vec<String>,
    /// Timeout (in seconds) per DNS request.
    #[serde(default = "default_dns_timeout")]
    pub timeout_secs: u64,
    /// Maximum number of concurrent DNS resolution tasks.
    #[serde(default = "default_dns_concurrency")]
    pub concurrency: usize,
}

fn default_countries() -> Vec<String> {
    vec!["de".to_string()]
}

fn default_dns_timeout() -> u64 {
    5
}

fn default_dns_concurrency() -> usize {
    64
}

/// Defines how monitoring results should be persisted.
#[derive(Clone, Debug, Deserialize)]
pub struct OutputConfig {
    /// Desired persistence target.
    #[serde(default = "OutputConfig::default_mode")]
    pub mode: OutputMode,
    /// File based configuration.
    #[serde(default)]
    pub file: Option<FileOutputConfig>,
    /// ClickHouse configuration.
    #[serde(default)]
    pub clickhouse: Option<ClickhouseConfig>,
}

impl OutputConfig {
    fn default_mode() -> OutputMode {
        OutputMode::File
    }
}

/// Output persistence modes supported by DNeyeS.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Persist events in NDJSON files on disk.
    File,
    /// Persist events directly into ClickHouse tables.
    Clickhouse,
    /// Persist events in both locations at the same time.
    Both,
}

/// Configuration for NDJSON exports.
#[derive(Clone, Debug, Deserialize)]
pub struct FileOutputConfig {
    /// Directory on disk where NDJSON exports are written to.
    #[serde(default = "default_export_directory")]
    pub directory: PathBuf,
}

fn default_export_directory() -> PathBuf {
    PathBuf::from("export")
}

/// Configuration for the ClickHouse backend.
#[derive(Clone, Debug, Deserialize)]
pub struct ClickhouseConfig {
    /// HTTP endpoint of the ClickHouse server, e.g. `http://localhost:8123`.
    pub url: String,
    /// Database name used to store DNeyeS tables.
    pub database: String,
    /// Username for authentication.
    #[serde(default = "default_clickhouse_user")]
    pub username: String,
    /// Password for authentication.
    #[serde(default)]
    pub password: String,
    /// Table that stores DNS resolution history.
    #[serde(default = "default_dns_table")]
    pub dns_table: String,
    /// Table that stores HTTP availability data.
    #[serde(default = "default_http_table")]
    pub http_table: String,
    /// Whether schema checks/creation should be attempted on start.
    #[serde(default = "default_clickhouse_schema_creation")]
    pub ensure_schema: bool,
    /// Optional request timeout in seconds.
    #[serde(default = "default_clickhouse_timeout")]
    pub timeout_secs: u64,
}

fn default_clickhouse_user() -> String {
    "default".to_string()
}

fn default_dns_table() -> String {
    "dns_resolutions".to_string()
}

fn default_http_table() -> String {
    "http_availability".to_string()
}

fn default_clickhouse_schema_creation() -> bool {
    true
}

fn default_clickhouse_timeout() -> u64 {
    10
}

/// REST API configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct ApiConfig {
    /// Bind address used by the API server.
    #[serde(default = "default_api_host")]
    pub host: String,
    /// Listening port of the API server.
    #[serde(default = "default_api_port")]
    pub port: u16,
    /// Default page size for API pagination.
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    /// Cross origin settings for the dashboard frontend.
    #[serde(default)]
    pub cors: CorsConfig,
}

fn default_api_host() -> String {
    "0.0.0.0".to_string()
}

fn default_api_port() -> u16 {
    8080
}

fn default_page_size() -> usize {
    100
}

/// Cross origin resource sharing configuration block.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct CorsConfig {
    /// List of allowed origins for browser applications.
    #[serde(default)]
    pub allow_origins: Vec<String>,
}
