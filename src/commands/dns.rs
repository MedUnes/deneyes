use std::fs;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::JoinSet;

use crate::config::AppConfig;
use crate::dns::dns_list_client::Client;
use crate::dns::dns_server::DnsServer;
use crate::errors::DneyesError;
use crate::telemetry::models::DnsResolutionEvent;
use crate::telemetry::sink::DnsSink;

/// Execute the DNS monitoring workflow.
pub(crate) async fn run(config: &AppConfig, sink: Arc<dyn DnsSink>) -> Result<(), DneyesError> {
    let file_contents = fs::read_to_string("sites.txt")?;

    let sites: Vec<String> = file_contents.lines().map(|line| line.to_string()).collect();
    let country_codes = &config.dns.default_countries;
    let timeout = config.dns.timeout_secs;

    let mut dns_servers_all_countries: Vec<DnsServer> = Vec::new();
    let client = Client::new(&config.dns.server_list_url);

    tracing::info!("Loading DNS server list", countries = ?country_codes);
    for country_code in country_codes {
        let dns_servers = client.fetch_dns_server_list(country_code).await?;
        tracing::info!(
            "Loaded DNS servers for country",
            country = country_code,
            count = dns_servers.len()
        );
        dns_servers_all_countries.extend(dns_servers.into_values());
    }

    tracing::info!(
        "DNS server catalogue loaded",
        total = dns_servers_all_countries.len()
    );

    let dns_servers: Vec<Arc<DnsServer>> = dns_servers_all_countries
        .into_iter()
        .map(Arc::new)
        .collect();

    let semaphore = Arc::new(Semaphore::new(config.dns.concurrency));
    let mut dns_futures = JoinSet::new();

    for site in &sites {
        for dns in &dns_servers {
            let domain_name = site.clone();
            let dns_server = dns.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let sink = sink.clone();

            dns_futures.spawn(async move {
                let _permit: OwnedSemaphorePermit = permit;
                let resolved_host = dns_server.resolv(domain_name.clone(), Some(timeout)).await;
                (dns_server, domain_name, resolved_host, sink)
            });
        }
    }

    while let Some(result) = dns_futures.join_next().await {
        match result {
            Ok((dns_server, domain_name, resolved_host_result, sink)) => match resolved_host_result
            {
                Ok(resolved_host) => {
                    let event: DnsResolutionEvent = resolved_host.into_event();
                    let has_resolved = event.resolved_ip.is_some();
                    let status_icon = if has_resolved { "✅" } else { "✖" };
                    tracing::info!(
                        "DNS resolution completed",
                        domain = domain_name,
                        dns_server = %dns_server.ip,
                        country = %dns_server.country_id,
                        duration_ms = event.duration_ms,
                        success = event.success
                    );
                    eprintln!(
                        "{} to {:?} with {} ...... {}({} ms)",
                        domain_name,
                        event.resolved_ip,
                        dns_server.name,
                        status_icon,
                        event.duration_ms,
                    );
                    sink.write(&event).await?;
                }
                Err(e) => {
                    tracing::error!(
                        "Error resolving domain",
                        domain = domain_name,
                        dns_server = %dns_server.ip,
                        error = %e
                    );
                    let event = DnsResolutionEvent::from_parts(
                        domain_name.clone(),
                        dns_server.ip.to_string(),
                        if dns_server.name.is_empty() {
                            None
                        } else {
                            Some(dns_server.name.clone())
                        },
                        dns_server.country_id.clone(),
                        None,
                        0,
                        Utc::now(),
                        Some(e.to_string()),
                    );
                    sink.write(&event).await?;
                }
            },
            Err(join_error) => {
                tracing::error!("Task join error: {}", join_error);
            }
        }
    }
    Ok(())
}
