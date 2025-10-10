use crate::telemetry::models::HttpAvailabilityEvent;
use crate::utils::time_utils::with_benchmark;
use chrono::Utc;

/// Execute an HTTP GET request against the provided base URL and capture timing
/// information.
pub async fn check(base_url: String) -> HttpAvailabilityEvent {
    let (result, duration) = with_benchmark(reqwest::get(base_url.clone())).await;
    match result {
        Ok(response) => HttpAvailabilityEvent {
            base_url,
            ip: response.remote_addr().map(|addr| addr.ip().to_string()),
            duration_ms: duration.num_milliseconds(),
            finished_at: Utc::now(),
            status_code: response.status().as_u16(),
            error: None,
        },
        Err(error) => HttpAvailabilityEvent {
            base_url,
            ip: None,
            duration_ms: duration.num_milliseconds(),
            finished_at: Utc::now(),
            status_code: 0,
            error: Some(error.to_string()),
        },
    }
}
