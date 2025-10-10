use thiserror::Error;

/// Custom error type that captures every failure that can occur in DNeyeS.
#[derive(Debug, Error)]
pub enum DneyesError {
    /// Errors that happen while (de-)serialising payloads to and from JSON.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// File system or I/O related errors.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Configuration loading/parsing failures.
    #[error("configuration error: {0}")]
    Config(String),
    /// Errors returned from the DNS resolver domain crate.
    #[error("dns error: {0}")]
    Dns(String),
    /// Errors happening while performing HTTP checks.
    #[error("http error: {0}")]
    Http(String),
    /// Errors originating from the ClickHouse client.
    #[error("clickhouse error: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),
    /// Errors originating from configuration framework.
    #[error("configuration error: {0}")]
    ConfigLoader(#[from] config::ConfigError),
}

impl From<reqwest::Error> for DneyesError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value.to_string())
    }
}
