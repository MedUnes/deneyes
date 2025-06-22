use crate::utils::time_utils::duration_deserializer;
use crate::utils::time_utils::duration_serializer;
use crate::utils::time_utils::with_benchmark;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::IpAddr;

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpSiteStatus {
    pub base_url: String,
    pub ip: Option<IpAddr>,
    #[serde(
        serialize_with = "duration_serializer",
        deserialize_with = "duration_deserializer"
    )]
    pub duration: chrono::Duration,
    pub finished_at: DateTime<Utc>,
    #[serde(
        serialize_with = "url_serializer",
        deserialize_with = "url_deserializer"
    )]
    pub redirect: Option<Url>,
    pub status_code: u16,
    pub error: Option<String>,
}
impl HttpSiteStatus {
    pub fn create(
        base_url: String,
        benchmarked_result: (Result<reqwest::Response, reqwest::Error>, chrono::Duration),
    ) -> Self {
        let (http_server, duration) = benchmarked_result;
        match http_server {
            Ok(http_server) => {
                let ip = http_server.remote_addr().unwrap().ip();
                HttpSiteStatus {
                    ip: Some(ip),
                    base_url,
                    duration,
                    finished_at: Utc::now(),
                    redirect: None,
                    status_code: http_server.status().as_u16(),
                    error: None,
                }
            }
            Err(error) => {
                println!("error: {}", error);
                HttpSiteStatus {
                    ip: None,
                    base_url,
                    duration,
                    finished_at: Utc::now(),
                    redirect: None,
                    status_code: 0,
                    error: Some(error.to_string()),
                }
            }
        }
    }
}

pub async fn check(base_url: String) -> HttpSiteStatus {
    let benchmarked_status = with_benchmark(reqwest::get(base_url.clone())).await;
    HttpSiteStatus::create(base_url, benchmarked_status)
}

fn url_serializer<S: Serializer>(_url: &Option<Url>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("n/a")
}
pub fn url_deserializer<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let url = Url::parse(&s);
    match url {
        Ok(url) => Ok(Some(url)),
        Err(_) => Ok(None),
    }
}
