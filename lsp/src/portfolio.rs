use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::Client;
use tracing::{debug, info, warn};

use crate::error::Result;

const THROTTLE_DURATION: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioPayload {
    pub workspace_name: String,
    pub file_name: String,
    pub language: String,
    pub line: u32,
    pub git_branch: Option<String>,
    pub git_remote: Option<String>,
}

#[derive(Debug)]
pub struct Portfolio {
    http_client: Client,
    endpoint_url: Option<String>,
    http_secret: Option<String>,
    shutting_down: Arc<AtomicBool>,
    last_payload: Option<PortfolioPayload>,
    last_sent_at: Option<Instant>,
}

impl Portfolio {
    pub fn new(shutting_down: Arc<AtomicBool>) -> Self {
        Self {
            http_client: Client::new(),
            endpoint_url: None,
            http_secret: None,
            shutting_down,
            last_payload: None,
            last_sent_at: None,
        }
    }

    pub fn configure(&mut self, endpoint_url: Option<String>, http_secret: Option<String>) {
        if let Some(ref url) = endpoint_url {
            info!("Portfolio endpoint configured: {}", url);
        } else {
            warn!("No endpoint_url in initialization_options");
        }

        self.endpoint_url = endpoint_url;
        self.http_secret = http_secret;
    }

    fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    pub async fn send_activity(&mut self, payload: &PortfolioPayload) -> Result<()> {
        if self.is_shutting_down() {
            debug!("Shutdown is in progress, skipping portfolio request");
            return Ok(());
        }

        let url = if let Some(u) = &self.endpoint_url {
            u.clone()
        } else {
            debug!("No endpoint_url configured, skipping portfolio request");
            return Ok(());
        };

        if self.last_payload.as_ref() == Some(payload) {
            debug!("Payload unchanged, skipping portfolio request");
            return Ok(());
        }

        if let Some(last_sent) = self.last_sent_at {
            let elapsed = last_sent.elapsed();

            if elapsed < THROTTLE_DURATION {
                debug!(
                    "Throttling portfolio request ({}ms since last send)",
                    elapsed.as_millis(),
                );
                return Ok(());
            }
        }

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));

        let body = serde_json::json!({
            "workspace": {
                "name": payload.workspace_name,
                "files": 0
            },
            "file": {
                "name": payload.file_name,
                "language": payload.language,
                "line": payload.line
            },
            "git": {
                "branch": payload.git_branch,
                "remote": payload.git_remote
            },
            "timestamp": timestamp_ms
        });

        let mut builder = self.http_client.put(&url).json(&body);

        if let Some(secret) = &self.http_secret {
            builder = builder.header("Authorization", format!("Bearer {secret}"));
        }

        let response = builder.send().await.map_err(|e| {
            warn!("Portfolio request failed: {}", e);
            crate::error::PresenceError::Http(e.to_string())
        })?;

        let status = response.status();

        if status.is_success() {
            info!("Portfolio activity sent (HTTP {})", status);
            self.last_payload = Some(payload.clone());
            self.last_sent_at = Some(Instant::now());
        } else {
            warn!("Portfolio API returned non-success status: {}", status);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shutdown_signal(value: bool) -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(value))
    }

    fn sample_payload() -> PortfolioPayload {
        PortfolioPayload {
            workspace_name: "my-project".to_string(),
            file_name: "main.rs".to_string(),
            language: "rust".to_string(),
            line: 42,
            git_branch: Some("main".to_string()),
            git_remote: Some("https://github.com/username/my-project".to_string()),
        }
    }

    #[test]
    fn test_new_defaults() {
        let portfolio = Portfolio::new(shutdown_signal(false));

        assert!(portfolio.last_payload.is_none());
    }

    #[tokio::test]
    async fn test_send_activity_skipped_during_shutdown() {
        let mut portfolio = Portfolio::new(shutdown_signal(true));
        portfolio.configure(Some("http://localhost:3000/activity".to_string()), None);

        assert!(portfolio.send_activity(&sample_payload()).await.is_ok());
        assert!(portfolio.last_payload.is_none());
    }

    #[tokio::test]
    async fn test_send_activity_skipped_without_endpoint() {
        let mut portfolio = Portfolio::new(shutdown_signal(false));

        assert!(portfolio.send_activity(&sample_payload()).await.is_ok());
        assert!(portfolio.last_payload.is_none());
    }
}
