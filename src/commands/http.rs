use std::fs;
use std::sync::Arc;

use tokio::task::JoinSet;

use crate::errors::DneyesError;
use crate::http::http_site_status;
use crate::telemetry::models::HttpAvailabilityEvent;
use crate::telemetry::sink::HttpSink;

/// Execute the HTTP monitoring workflow.
pub(crate) async fn run(sink: Arc<dyn HttpSink>) -> Result<(), DneyesError> {
    let file_contents = fs::read_to_string("sites.txt")?;
    let sites: Vec<String> = file_contents.lines().map(|line| line.to_string()).collect();
    let mut https_futures = JoinSet::new();

    for site in &sites {
        let site_clone = format!("https://{}", site.clone());
        let sink = sink.clone();
        https_futures.spawn(async move {
            let event: HttpAvailabilityEvent = http_site_status::check(site_clone.clone()).await;
            (site_clone, event, sink)
        });
    }

    while let Some(result) = https_futures.join_next().await {
        match result {
            Ok((site, event, sink)) => {
                let success = event.error.is_none();
                let status_icon = if success { "✅" } else { "✖" };
                eprintln!(
                    "{} Responded successfully: {} ...... ({} ms)",
                    site, status_icon, event.duration_ms,
                );
                sink.write(&event).await?;
            }
            Err(e) => {
                tracing::error!("Error checking HTTP status: {}", e);
            }
        }
    }

    Ok(())
}
