use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};

/// Serialize a [`DateTime`] instance using RFC3339 formatting.
pub fn zulu_serializer<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted_date = datetime.to_rfc3339();
    serializer.serialize_str(&formatted_date)
}

/// Deserialize an RFC3339 formatted string into a [`DateTime`].
pub fn zulu_deserializer<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let datetime = DateTime::parse_from_rfc3339(&s)
        .expect("Unable to Parse zulu formatted date/time")
        .to_utc();
    Ok(datetime)
}

/// Serialize a [`chrono::Duration`] into microseconds.
pub fn duration_serializer<S>(duration: &chrono::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(duration.num_microseconds().unwrap_or(-1))
}

/// Deserialize a [`chrono::Duration`] from microseconds.
pub fn duration_deserializer<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let duration_microseconds = i64::deserialize(deserializer)?;
    Ok(chrono::Duration::microseconds(duration_microseconds))
}

/// Benchmark a future by returning the result alongside the elapsed time.
pub async fn with_benchmark<F, T, E>(fut: F) -> (Result<T, E>, chrono::Duration)
where
    F: std::future::Future<Output = Result<T, E>>,
{
    let start = Utc::now();
    let result = fut.await;
    let end = Utc::now();
    let duration = end.signed_duration_since(start) as chrono::Duration;

    (result, duration)
}
