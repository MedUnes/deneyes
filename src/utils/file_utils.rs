use chrono::Utc;
use std::fmt;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

pub enum StatusFileType {
    DNS,
    HTTP,
}
impl fmt::Display for StatusFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatusFileType::DNS => write!(f, "{}", "dns"),
            StatusFileType::HTTP => write!(f, "{}", "http"),
        }
    }
}
pub struct StatusFile {
    pub file: File,
    pub status_type: StatusFileType,
}
impl StatusFile {
    pub async fn create(status_type: StatusFileType) -> Self {
        let now = Utc::now().timestamp().to_string();
        let file_name = format!("export/dneyes_status_{status_type}_{now}.ndjson");
        println!("{}", file_name);
        let file = OpenOptions::new()
            .append(true)
            .read(true)
            .write(true)
            .create(true)
            .open(file_name)
            .await
            .expect("Couldn't open radar status file");
        StatusFile { status_type, file }
    }
    pub async fn write(&mut self, data: &[u8]) {
        self.file.write_all(data).await.unwrap()
    }
    pub async fn close(&mut self) {
        self.file.flush().await.unwrap()
    }
}
