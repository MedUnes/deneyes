#[derive(Debug)]
pub enum DneyesError {
    Serialization(serde_json::error::Error),
    Io(std::io::Error),
    Dns(String),
    Http(String),
}

impl std::error::Error for DneyesError {}

impl std::fmt::Display for DneyesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DneyesError::Serialization(e) => write!(f, "IO error: {}", e),
            DneyesError::Io(e) => write!(f, "IO error: {}", e),
            DneyesError::Dns(e) => write!(f, "DNS error: {}", e),
            DneyesError::Http(e) => write!(f, "HTTP error: {}", e),
        }
    }
}
