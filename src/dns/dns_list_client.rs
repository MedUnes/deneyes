use crate::dns::dns_server::DnsServer;
use crate::errors::DneyesError;
use std::collections::HashMap;

/// Minimal HTTP client responsible for downloading the public DNS resolver
/// list for the configured set of countries.
pub struct Client {
    base_url: String,
}

impl Client {
    /// Create a new client with the provided base URL, e.g.
    /// `https://public-dns.info/nameserver`.
    pub fn new(base_url: &str) -> Client {
        Client {
            base_url: base_url.to_string(),
        }
    }

    /// Fetch and parse the DNS server JSON payload for a given ISO country code.
    pub async fn fetch_dns_server_list(
        &self,
        country_code: &str,
    ) -> Result<HashMap<String, DnsServer>, DneyesError> {
        let url = format!("{}/{}.json", self.base_url, country_code);
        let dns_server_list = reqwest::get(url)
            .await
            .map_err(|e| DneyesError::Dns(e.to_string()))?
            .json::<Vec<DnsServer>>()
            .await
            .map_err(|e| DneyesError::Dns(e.to_string()))?;

        Ok(dns_server_list
            .into_iter()
            .map(|dns_server| (dns_server.ip.to_string(), dns_server))
            .collect())
    }
}
