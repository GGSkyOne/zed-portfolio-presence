use crate::{
    document::Document, error::Result, idle::IdleManager, languages::get_language,
    portfolio::PortfolioPayload, service::AppState,
};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug)]
pub struct PresenceService {
    state: Arc<AppState>,
    idle_manager: IdleManager,
}

impl PresenceService {
    pub fn new(state: Arc<AppState>) -> Self {
        let idle_manager = IdleManager::new(Arc::clone(&state.shutting_down));

        Self {
            state,
            idle_manager,
        }
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

        if doc.is_some() {
            self.reset_idle_timeout().await?;
        }

        if self.state.is_shutting_down() {
            debug!("Shutdown is in progress, skipping portfolio request");
            return Ok(());
        }

        self.send_portfolio_activity(doc.as_ref()).await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if !self.state.mark_shutting_down() {
            debug!("Presence service shutdown already in progress");
            self.idle_manager.cancel_timeout().await;
            return Ok(());
        }

        self.idle_manager.cancel_timeout().await;

        Ok(())
    }

    async fn send_portfolio_activity(&self, doc: Option<&Document>) -> Result<()> {
        if self.state.is_shutting_down() {
            debug!("Shutdown is in progress, skipping portfolio request");
            return Ok(());
        }

        let doc = match doc {
            Some(d) => d,
            None => {
                debug!("No document available, skipping portfolio request");
                return Ok(());
            }
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

    async fn reset_idle_timeout(&self) -> Result<()> {
        if self.state.is_shutting_down() {
            debug!("Skipping idle timeout reset because shutdown is in progress");
            return Ok(());
        }

        let workspace_name = {
            let workspace = self.state.workspace.lock().await;
            workspace.name().to_string()
        };

        self.idle_manager
            .reset_timeout(
                Arc::clone(&self.state.portfolio),
                Arc::clone(&self.state.config),
                workspace_name,
            )
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_cancels_idle_timeout_and_marks_state() {
        let state = Arc::new(AppState::new());
        let service = PresenceService::new(Arc::clone(&state));

        service.reset_idle_timeout().await.unwrap();
        assert!(service.idle_manager.has_timeout().await);

        service.shutdown().await.unwrap();

        assert!(state.is_shutting_down());
        assert!(!service.idle_manager.has_timeout().await);
    }

    #[tokio::test]
    async fn test_shutdown_is_idempotent() {
        let state = Arc::new(AppState::new());
        let service = PresenceService::new(state);

        assert!(service.shutdown().await.is_ok());
        assert!(service.shutdown().await.is_ok());
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
