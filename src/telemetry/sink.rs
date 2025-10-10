use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::config::{OutputConfig, OutputMode};
use crate::errors::DneyesError;
use crate::telemetry::clickhouse::{ClickhouseContext, DnsQueryFilter};
use crate::telemetry::models::{DnsResolutionEvent, HttpAvailabilityEvent};

/// Trait implemented by every DNS sink.
#[async_trait]
pub trait DnsSink: Send + Sync {
    /// Persist a DNS resolution event.
    async fn write(&self, event: &DnsResolutionEvent) -> Result<(), DneyesError>;
}

/// Trait implemented by HTTP sinks.
#[async_trait]
pub trait HttpSink: Send + Sync {
    /// Persist an HTTP availability event.
    async fn write(&self, event: &HttpAvailabilityEvent) -> Result<(), DneyesError>;
}

/// Helper struct bundling DNS and HTTP sinks.
#[derive(Clone)]
pub struct SinkSet {
    /// Sink used to persist DNS events.
    pub dns: Arc<dyn DnsSink>,
    /// Sink used to persist HTTP events.
    pub http: Arc<dyn HttpSink>,
    /// Optional ClickHouse context exposed for the REST API.
    pub clickhouse: Option<ClickhouseContext>,
}

/// Construct sinks based on the loaded configuration.
pub async fn build_sinks(config: &OutputConfig) -> Result<SinkSet, DneyesError> {
    let mut dns_sinks: Vec<Arc<dyn DnsSink>> = Vec::new();
    let mut http_sinks: Vec<Arc<dyn HttpSink>> = Vec::new();
    let mut clickhouse: Option<ClickhouseContext> = None;

    if matches!(config.mode, OutputMode::File | OutputMode::Both) {
        if let Some(file_config) = &config.file {
            let dns_file_sink = Arc::new(FileDnsSink::new(&file_config.directory).await?);
            let http_file_sink = Arc::new(FileHttpSink::new(&file_config.directory).await?);
            dns_sinks.push(dns_file_sink);
            http_sinks.push(http_file_sink);
        }
    }

    if matches!(config.mode, OutputMode::Clickhouse | OutputMode::Both) {
        if let Some(clickhouse_config) = &config.clickhouse {
            let ctx = ClickhouseContext::new(clickhouse_config).await?;
            let dns_clickhouse_sink = Arc::new(ClickhouseDnsSink {
                context: ctx.clone(),
            });
            let http_clickhouse_sink = Arc::new(ClickhouseHttpSink {
                context: ctx.clone(),
            });
            dns_sinks.push(dns_clickhouse_sink);
            http_sinks.push(http_clickhouse_sink);
            clickhouse = Some(ctx);
        }
    }

    let dns = if dns_sinks.len() == 1 {
        dns_sinks.pop().unwrap()
    } else {
        Arc::new(CompositeDnsSink { sinks: dns_sinks })
    };

    let http = if http_sinks.len() == 1 {
        http_sinks.pop().unwrap()
    } else {
        Arc::new(CompositeHttpSink { sinks: http_sinks })
    };

    Ok(SinkSet {
        dns,
        http,
        clickhouse,
    })
}

struct FileWriter {
    file: Arc<Mutex<tokio::fs::File>>,
}

impl FileWriter {
    async fn new(directory: &Path, prefix: &str) -> Result<Self, DneyesError> {
        tokio::fs::create_dir_all(directory).await?;
        let now = Utc::now().format("%Y%m%d%H%M%S");
        let file_name = directory.join(format!("dneyes_{prefix}_{now}.ndjson"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)
            .await?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    async fn write(&self, data: &[u8]) -> Result<(), DneyesError> {
        let mut guard = self.file.lock().await;
        guard.write_all(data).await?;
        guard.write_all(b"\n").await?;
        guard.flush().await?;
        Ok(())
    }
}

struct FileDnsSink {
    writer: FileWriter,
}

impl FileDnsSink {
    async fn new(directory: &Path) -> Result<Self, DneyesError> {
        Ok(Self {
            writer: FileWriter::new(directory, "dns").await?,
        })
    }
}

#[async_trait]
impl DnsSink for FileDnsSink {
    async fn write(&self, event: &DnsResolutionEvent) -> Result<(), DneyesError> {
        let payload = serde_json::to_vec(event)?;
        self.writer.write(&payload).await
    }
}

struct FileHttpSink {
    writer: FileWriter,
}

impl FileHttpSink {
    async fn new(directory: &Path) -> Result<Self, DneyesError> {
        Ok(Self {
            writer: FileWriter::new(directory, "http").await?,
        })
    }
}

#[async_trait]
impl HttpSink for FileHttpSink {
    async fn write(&self, event: &HttpAvailabilityEvent) -> Result<(), DneyesError> {
        let payload = serde_json::to_vec(event)?;
        self.writer.write(&payload).await
    }
}

struct ClickhouseDnsSink {
    context: ClickhouseContext,
}

#[async_trait]
impl DnsSink for ClickhouseDnsSink {
    async fn write(&self, event: &DnsResolutionEvent) -> Result<(), DneyesError> {
        self.context.insert_dns(event).await
    }
}

struct ClickhouseHttpSink {
    context: ClickhouseContext,
}

#[async_trait]
impl HttpSink for ClickhouseHttpSink {
    async fn write(&self, event: &HttpAvailabilityEvent) -> Result<(), DneyesError> {
        self.context.insert_http(event).await
    }
}

struct CompositeDnsSink {
    sinks: Vec<Arc<dyn DnsSink>>,
}

#[async_trait]
impl DnsSink for CompositeDnsSink {
    async fn write(&self, event: &DnsResolutionEvent) -> Result<(), DneyesError> {
        for sink in &self.sinks {
            sink.write(event).await?;
        }
        Ok(())
    }
}

struct CompositeHttpSink {
    sinks: Vec<Arc<dyn HttpSink>>,
}

#[async_trait]
impl HttpSink for CompositeHttpSink {
    async fn write(&self, event: &HttpAvailabilityEvent) -> Result<(), DneyesError> {
        for sink in &self.sinks {
            sink.write(event).await?;
        }
        Ok(())
    }
}
