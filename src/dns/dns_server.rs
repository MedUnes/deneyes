use crate::errors::DneyesError;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsServer {
    pub ip: IpAddr,
    pub name: String,
    as_number: u32,
    as_org: String,
    pub country_id: String,
    pub city: String,
    version: String,
    error: String,
    dnssec: bool,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvedHost {
    pub dns_server_ip: IpAddr,
    
    pub fqdn: String,
    pub ip: Option<IpAddr>,
    #[serde(
        serialize_with = "duration_serializer",
        deserialize_with = "duration_deserializer"
    )]
    pub duration: chrono::Duration,
    pub finished_at: DateTime<Utc>,
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
                    fqdn,
                    ip,
                    duration,
                    finished_at: Utc::now(),
                    error: None,
                }
            }
            Err(error) => ResolvedHost {
                dns_server_ip: dns_server.ip,
                fqdn,
                ip: None,
                duration,
                finished_at: Utc::now(),
                error: Some(error.to_string()),
            },
        }
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
        assert_eq!(
            resolved_host.ip.unwrap().to_string().as_str(),
            "142.250.185.78"
        );
    }
}
