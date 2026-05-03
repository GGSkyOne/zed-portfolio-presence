use crate::{
    document::Document, error::Result, languages::get_language, portfolio::PortfolioPayload,
    service::AppState,
};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug)]
pub struct PresenceService {
    state: Arc<AppState>,
}

impl PresenceService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn update_presence(&self, doc: Option<Document>) -> Result<()> {
        if self.state.is_shutting_down() {
            debug!("Shutdown is in progress, skipping portfolio request");
            return Ok(());
        }

        {
            let mut last_doc = self.state.last_document.lock().await;
            (*last_doc).clone_from(&doc);
        }

        if self.state.is_shutting_down() {
            debug!("Shutdown is in progress, skipping portfolio request");
            return Ok(());
        }

        self.send_portfolio_activity(doc.as_ref()).await?;

        Ok(())
    }

    pub fn shutdown(&self) {
        if !self.state.mark_shutting_down() {
            debug!("Presence service shutdown already in progress");
        }
    }

    async fn send_portfolio_activity(&self, doc: Option<&Document>) -> Result<()> {
        if self.state.is_shutting_down() {
            debug!("Shutdown is in progress, skipping portfolio request");
            return Ok(());
        }

        let Some(doc) = doc else {
            debug!("No document available, skipping portfolio request");
            return Ok(());
        };

        let (workspace_name, git_enabled) = {
            let config = self.state.config.lock().await;
            let workspace = self.state.workspace.lock().await;
            (workspace.name().to_string(), config.git_integration)
        };

        let git_branch = self.state.git_branch.lock().await.clone();
        let git_remote = if git_enabled {
            self.state.git_remote_url.lock().await.clone()
        } else {
            None
        };

        let payload = PortfolioPayload {
            workspace_name,
            file_name: doc.get_filename().unwrap_or_else(|_| "unknown".to_string()),
            language: get_language(doc),
            line: doc.get_line_number().unwrap_or(0),
            git_branch,
            git_remote,
        };

        let mut portfolio = self.state.portfolio.lock().await;
        portfolio.send_activity(&payload).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_is_idempotent() {
        let state = Arc::new(AppState::new());
        let service = PresenceService::new(state);

        service.shutdown();
        service.shutdown();
    }

    #[tokio::test]
    async fn test_update_presence_is_ignored_during_shutdown() {
        let state = Arc::new(AppState::new());
        let service = PresenceService::new(Arc::clone(&state));

        assert!(state.mark_shutting_down());
        assert!(service.update_presence(None).await.is_ok());

        let last_document = state.last_document.lock().await;
        assert!(last_document.is_none());
    }
}
