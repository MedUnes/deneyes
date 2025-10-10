use chrono::{DateTime, Utc};
use clickhouse::Client;
use tokio::sync::Mutex;

use crate::config::ClickhouseConfig;
use crate::errors::DneyesError;
use crate::telemetry::models::{DnsResolutionEvent, HttpAvailabilityEvent};

/// Wrapper around the ClickHouse client that encapsulates schema management and
/// query helpers used throughout the application.
#[derive(Clone)]
pub struct ClickhouseContext {
    client: Client,
    config: ClickhouseConfig,
    insert_lock: std::sync::Arc<Mutex<()>>,
}

impl ClickhouseContext {
    /// Create a new ClickHouse context and ensure that required tables exist.
    pub async fn new(config: &ClickhouseConfig) -> Result<Self, DneyesError> {
        let base_client = Client::default()
            .with_url(&config.url)
            .with_user(&config.username)
            .with_password(&config.password);

        if config.ensure_schema {
            base_client
                .query(format!("CREATE DATABASE IF NOT EXISTS {}", config.database))
                .execute()
                .await?;
        }

        let client = base_client.clone().with_database(&config.database);

        if config.ensure_schema {
            let dns_table_sql = format!(
                "CREATE TABLE IF NOT EXISTS {table} (
                    fqdn String,
                    dns_server_ip String,
                    dns_server_name Nullable(String),
                    dns_server_country LowCardinality(String),
                    resolved_ip Nullable(String),
                    success Bool,
                    duration_ms Int64,
                    finished_at DateTime64(3, 'UTC'),
                    error Nullable(String)
                )
                ENGINE = MergeTree()
                ORDER BY (fqdn, finished_at)",
                table = config.dns_table,
            );
            client.query(dns_table_sql).execute().await?;

            let http_table_sql = format!(
                "CREATE TABLE IF NOT EXISTS {table} (
                    base_url String,
                    ip Nullable(String),
                    duration_ms Int64,
                    finished_at DateTime64(3, 'UTC'),
                    status_code UInt16,
                    error Nullable(String)
                )
                ENGINE = MergeTree()
                ORDER BY (base_url, finished_at)",
                table = config.http_table,
            );
            client.query(http_table_sql).execute().await?;
        }

        Ok(Self {
            client,
            config: config.clone(),
            insert_lock: std::sync::Arc::new(Mutex::new(())),
        })
    }

    /// Insert a DNS resolution event into ClickHouse.
    pub async fn insert_dns(&self, event: &DnsResolutionEvent) -> Result<(), DneyesError> {
        let _guard = self.insert_lock.lock().await;
        let mut insert = self.client.insert(self.config.dns_table.clone())?;
        insert.write(event).await?;
        insert.end().await?;
        Ok(())
    }

    /// Insert an HTTP availability event into ClickHouse.
    pub async fn insert_http(&self, event: &HttpAvailabilityEvent) -> Result<(), DneyesError> {
        let _guard = self.insert_lock.lock().await;
        let mut insert = self.client.insert(self.config.http_table.clone())?;
        insert.write(event).await?;
        insert.end().await?;
        Ok(())
    }

    /// Fetch a filtered set of DNS resolution events for the REST API.
    pub async fn query_dns(
        &self,
        filter: &DnsQueryFilter,
        limit: usize,
    ) -> Result<Vec<DnsResolutionEvent>, DneyesError> {
        let mut sql = format!(
            "SELECT fqdn, dns_server_ip, dns_server_name, dns_server_country, resolved_ip, success, duration_ms, finished_at, error FROM {}",
            self.config.dns_table
        );
        let mut conditions: Vec<&str> = Vec::new();
        if filter.domain.is_some() {
            conditions.push("fqdn = ?");
        }
        if filter.country.is_some() {
            conditions.push("dns_server_country = ?");
        }
        if filter.success.is_some() {
            conditions.push("success = ?");
        }
        if filter.from.is_some() {
            conditions.push("finished_at >= ?");
        }
        if filter.to.is_some() {
            conditions.push("finished_at <= ?");
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY finished_at DESC LIMIT ?");

        let mut query = self.client.query(sql);
        if let Some(domain) = &filter.domain {
            query = query.bind(domain);
        }
        if let Some(country) = &filter.country {
            query = query.bind(country);
        }
        if let Some(success) = filter.success {
            query = query.bind(success);
        }
        if let Some(from) = &filter.from {
            query = query.bind(from);
        }
        if let Some(to) = &filter.to {
            query = query.bind(to);
        }
        query = query.bind(limit as u64);

        let rows: Vec<DnsResolutionEvent> = query.fetch_all().await?;
        Ok(rows)
    }

    /// Simple health check to validate that ClickHouse is reachable.
    pub async fn ping(&self) -> Result<(), DneyesError> {
        self.client.query("SELECT 1").execute().await?;
        Ok(())
    }
}

/// Query filter used by the REST API to fetch DNS data from ClickHouse.
#[derive(Clone, Debug, Default)]
pub struct DnsQueryFilter {
    /// Domain name to filter for.
    pub domain: Option<String>,
    /// Filter by resolver country.
    pub country: Option<String>,
    /// Filter by success state.
    pub success: Option<bool>,
    /// Fetch records newer than this timestamp.
    pub from: Option<DateTime<Utc>>,
    /// Fetch records older than this timestamp.
    pub to: Option<DateTime<Utc>>,
}
