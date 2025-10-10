use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Normalised DNS resolution event used by the persistence sinks and the REST
/// API layer.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DnsResolutionEvent {
    /// Fully qualified domain name that has been resolved.
    pub fqdn: String,
    /// IP address of the DNS resolver used for this lookup.
    pub dns_server_ip: String,
    /// Human readable DNS server name when available.
    #[schema(example = "Google Public DNS")]
    pub dns_server_name: Option<String>,
    /// ISO country identifier of the DNS server.
    #[schema(example = "de")]
    pub dns_server_country: String,
    /// IP address returned by the DNS server.
    pub resolved_ip: Option<String>,
    /// Indicates whether the lookup succeeded.
    pub success: bool,
    /// How long the lookup took in milliseconds.
    pub duration_ms: i64,
    /// Timestamp when the lookup finished.
    pub finished_at: DateTime<Utc>,
    /// Optional error message in case of failures.
    pub error: Option<String>,
}

impl DnsResolutionEvent {
    /// Convenience helper to derive the `success` flag from an error optional.
    pub fn from_parts(
        fqdn: String,
        dns_server_ip: String,
        dns_server_name: Option<String>,
        dns_server_country: String,
        resolved_ip: Option<String>,
        duration_ms: i64,
        finished_at: DateTime<Utc>,
        error: Option<String>,
    ) -> Self {
        let success = error.is_none() && resolved_ip.is_some();
        Self {
            fqdn,
            dns_server_ip,
            dns_server_name,
            dns_server_country,
            resolved_ip,
            success,
            duration_ms,
            finished_at,
            error,
        }
    }
}

/// Normalised HTTP availability event.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct HttpAvailabilityEvent {
    /// Base URL of the monitored HTTP endpoint.
    pub base_url: String,
    /// IP address that served the request (if known).
    pub ip: Option<String>,
    /// Request latency in milliseconds.
    pub duration_ms: i64,
    /// Timestamp when the check completed.
    pub finished_at: DateTime<Utc>,
    /// HTTP status code returned by the server.
    pub status_code: u16,
    /// Optional error details.
    pub error: Option<String>,
}
