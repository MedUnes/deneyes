use crate::errors::DneyesError;
use crate::telemetry::models::DnsResolutionEvent;
use crate::utils::time_utils::*;
use chrono::{DateTime, Utc};

use domain::resolv::stub::conf::Transport::UdpTcp;
use domain::{
    base::Name,
    resolv::{
        lookup::host::FoundHosts,
        stub::conf::{ResolvConf, ServerConf},
        StubResolver,
    },
};
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

/// Representation of a DNS resolver as returned from the public DNS catalogue.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsServer {
    /// IP address of the resolver.
    pub ip: IpAddr,
    /// Human readable name if provided by the catalogue.
    pub name: String,
    as_number: u32,
    as_org: String,
    /// ISO country identifier of the resolver.
    pub country_id: String,
    /// City of the resolver (optional in the public dataset).
    pub city: String,
    version: String,
    error: String,
    dnssec: bool,
    /// Reported resolver reliability.
    pub reliability: f32,
    #[serde(
        serialize_with = "zulu_serializer",
        deserialize_with = "zulu_deserializer"
    )]
    checked_at: DateTime<Utc>,
    #[serde(
        serialize_with = "zulu_serializer",
        deserialize_with = "zulu_deserializer"
    )]
    created_at: DateTime<Utc>,
}

impl FromStr for DnsServer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DnsServer {
            ip: IpAddr::from_str(s).expect("Invalid IP address"),
            name: "".to_string(),
            as_number: 0,
            as_org: "".to_string(),
            country_id: "us".to_string(),
            city: "".to_string(),
            version: "".to_string(),
            error: "".to_string(),
            dnssec: false,
            reliability: 0.5,
            checked_at: Utc::now(),
            created_at: Utc::now(),
        })
    }
}

impl DnsServer {
    /// Resolve the provided domain name using this resolver.
    pub(crate) async fn resolv(
        &self,
        name: String,
        timeout: Option<u64>,
    ) -> Result<ResolvedHost, DneyesError> {
        let mut resolv_conf = ResolvConf::default();
        let server_conf = ServerConf::new(
            SocketAddr::new(
                IpAddr::from_str(self.ip.to_string().as_str())
                    .map_err(|e| DneyesError::Dns(e.to_string()))?,
                53,
            ),
            UdpTcp,
        );
        resolv_conf.servers.clear();
        resolv_conf.servers.push(server_conf);
        resolv_conf.options.timeout = Duration::new(timeout.unwrap_or(5), 0);
        resolv_conf.options.attempts = 1;

        let resolver = StubResolver::from_conf(resolv_conf);
        let qname: Name<Vec<u8>> =
            Name::from_str(&name).map_err(|e| DneyesError::Dns(e.to_string()))?;
        let benchmarked_result = with_benchmark(resolver.lookup_host(qname));

        Ok(ResolvedHost::create(
            name.to_string(),
            self,
            benchmarked_result.await,
        ))
    }
}

/// DNS resolution output generated from [`DnsServer::resolv`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedHost {
    /// DNS server IP that answered the query.
    pub dns_server_ip: IpAddr,
    /// Name of the DNS server.
    pub dns_server_name: Option<String>,
    /// Country of the DNS server.
    pub dns_server_country: String,
    /// Requested FQDN.
    pub fqdn: String,
    /// Resolved IP, if any.
    pub ip: Option<IpAddr>,
    #[serde(
        serialize_with = "duration_serializer",
        deserialize_with = "duration_deserializer"
    )]
    /// Duration of the lookup.
    pub duration: chrono::Duration,
    /// Timestamp when the lookup finished.
    pub finished_at: DateTime<Utc>,
    /// Optional error that occurred.
    pub error: Option<String>,
}

impl ResolvedHost {
    pub fn create(
        fqdn: String,
        dns_server: &DnsServer,
        benchmarked_result: (Result<FoundHosts<&StubResolver>, Error>, chrono::Duration),
    ) -> Self {
        let (resolved_result, duration) = benchmarked_result;
        match resolved_result {
            Ok(resolved_hosts) => {
                let ip = resolved_hosts
                    .iter()
                    .map(|ip_addr| ip_addr.to_canonical())
                    .last();

                ResolvedHost {
                    dns_server_ip: dns_server.ip,
                    dns_server_name: if dns_server.name.is_empty() {
                        None
                    } else {
                        Some(dns_server.name.clone())
                    },
                    dns_server_country: dns_server.country_id.clone(),
                    fqdn,
                    ip,
                    duration,
                    finished_at: Utc::now(),
                    error: None,
                }
            }
            Err(error) => ResolvedHost {
                dns_server_ip: dns_server.ip,
                dns_server_name: if dns_server.name.is_empty() {
                    None
                } else {
                    Some(dns_server.name.clone())
                },
                dns_server_country: dns_server.country_id.clone(),
                fqdn,
                ip: None,
                duration,
                finished_at: Utc::now(),
                error: Some(error.to_string()),
            },
        }
    }

    /// Convert this resolution output into the normalised event structure used
    /// by persistence sinks.
    pub fn into_event(self) -> DnsResolutionEvent {
        DnsResolutionEvent::from_parts(
            self.fqdn,
            self.dns_server_ip.to_string(),
            self.dns_server_name,
            self.dns_server_country,
            self.ip.map(|ip| ip.to_string()),
            self.duration.num_milliseconds(),
            self.finished_at,
            self.error,
        )
    }
}

mod test {
    #[cfg(test)]
    #[tokio::test]
    pub async fn test_resolv() {
        use super::*;

        let dns_server = DnsServer::from_str("8.8.8.8").unwrap();

        let resolved_host = dns_server
            .resolv("google.com".to_string(), None)
            .await
            .unwrap();
        assert_eq!(resolved_host.fqdn, "google.com".to_string());
        assert!(resolved_host.ip.is_some());
    }
}
