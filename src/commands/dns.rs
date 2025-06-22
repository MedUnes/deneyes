use crate::dns::dns_list_client::Client;
use crate::dns::dns_server::DnsServer;
use crate::errors::DneyesError;
use crate::utils::file_utils::{StatusFile, StatusFileType};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::task::JoinSet;

pub(crate) async fn run() -> Result<(), DneyesError> {
    let file_contents = fs::read_to_string("sites.txt");

    let sites: Vec<String> = file_contents
        .expect("error")
        .lines()
        .map(|line| line.to_string())
        .collect();

    let country_codes = vec!["de"];
    let mut status_file_dns = StatusFile::create(StatusFileType::DNS).await;
    let client = Client::new();

    println!("Loading fresh DNS server list..\n");
    let mut dns_servers_all_countries: HashMap<String, DnsServer> = HashMap::new();
    for country_code in &country_codes {
        let dns_servers = client.fetch_dns_server_list(*country_code).await?;
        println!("{}.. ({} servers) ✅ \n", country_code, dns_servers.len());
        dns_servers_all_countries.extend(dns_servers);
    }
    println!(
        "DNS server list fully loaded..({} servers) ✅ \n",
        dns_servers_all_countries.len()
    );

    let dns_servers: Vec<Arc<DnsServer>> = dns_servers_all_countries
        .into_values()
        .map(Arc::new)
        .collect();
    ();
    let mut dns_futures = JoinSet::new();

    for site in &sites {
        for dns in &dns_servers {
            let dns_server = dns.clone();
            let domain_name = site.clone(); // Already a String, clone is cheap
            let dns_fut = async move {
                let resolved_host = dns_server.resolv(domain_name.clone(), Some(5)).await;
                (dns_server, domain_name, resolved_host)
            };
            dns_futures.spawn(dns_fut);
        }
    }

    while let Some(result) = dns_futures.join_next().await {
        match result {
            Ok((dns_server, domain_name, resolved_host_result)) => match resolved_host_result {
                Ok(resolved_host) => {
                    let has_resolved = resolved_host.ip.is_some();
                    let status_icon = if has_resolved { "✅" } else { "✖" };
                    let ip = resolved_host
                        .ip
                        .as_ref()
                        .map(|ip| ip.to_string())
                        .unwrap_or_else(|| "n/a".to_string());
                    eprint!(
                        "{} to {} with {} ...... {}({} ms)\n",
                        domain_name,
                        ip,
                        resolved_host.dns_server_ip,
                        status_icon,
                        resolved_host.duration.num_milliseconds(),
                    );
                    status_file_dns
                        .write(
                            serde_json::to_string(&resolved_host)
                                .map_err(DneyesError::Serialization)?
                                .as_bytes(),
                        )
                        .await;
                }
                Err(e) => {
                    eprintln!(
                        "Error resolving {} with {}: {}",
                        domain_name, dns_server.name, e
                    );
                }
            },
            Err(join_error) => {
                eprintln!("Task join error: {}", join_error);
            }
        }
    }
    Ok(())
}
