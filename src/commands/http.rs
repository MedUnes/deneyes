use crate::errors::DneyesError;
use crate::http::http_site_status;
use crate::utils::file_utils::{StatusFile, StatusFileType};
use std::fs;
use tokio::task::JoinSet;

pub(crate) async fn run() -> Result<(), DneyesError> {
    let file_contents = fs::read_to_string("sites.txt");
    let sites: Vec<String> = file_contents
        .expect("error")
        .lines()
        .map(|line| line.to_string())
        .collect();
    let mut status_file_http = StatusFile::create(StatusFileType::HTTP).await;
    let mut https_futures = JoinSet::new();

    for site in &sites {
        let site_clone = format!("https://{}", site.clone());
        let https_fut = async move {
            let http_site_status = http_site_status::check(site_clone.clone()).await;
            (site_clone, http_site_status)
        };
        https_futures.spawn(https_fut);

        while let Some(result) = https_futures.join_next().await {
            match result {
                Ok((site, http_site_status)) => {
                    let success = !http_site_status.error.is_some();
                    let status_icon = if success { "✅" } else { "✖" };
                    eprint!(
                        "{} Responded successfully: {} ...... ({} ms)\n",
                        site,
                        status_icon,
                        http_site_status.duration.num_milliseconds(),
                    );
                    status_file_http
                        .write(
                            serde_json::to_string(&http_site_status)
                                .map_err(|e| DneyesError::Serialization(e))?
                                .as_bytes(),
                        )
                        .await;
                }
                Err(e) => {
                    eprintln!("Error checking status {}", e);
                }
            }
        }
    }
    Ok(())
}
