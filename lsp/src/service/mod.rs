mod presence_service;
mod workspace_service;

pub use presence_service::PresenceService;
pub use workspace_service::WorkspaceService;

use crate::{config::Configuration, document::Document, portfolio::Portfolio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub portfolio: Arc<Mutex<Portfolio>>,
    pub config: Arc<Mutex<Configuration>>,
    pub workspace: Arc<Mutex<WorkspaceService>>,
    pub git_remote_url: Arc<Mutex<Option<String>>>,
    pub git_branch: Arc<Mutex<Option<String>>>,
    pub last_document: Arc<Mutex<Option<Document>>>,
    pub shutting_down: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        let shutting_down = Arc::new(AtomicBool::new(false));

        Self {
            portfolio: Arc::new(Mutex::new(Portfolio::new(Arc::clone(&shutting_down)))),
            config: Arc::new(Mutex::new(Configuration::default())),
            workspace: Arc::new(Mutex::new(WorkspaceService::new())),
            git_remote_url: Arc::new(Mutex::new(None)),
            git_branch: Arc::new(Mutex::new(None)),
            last_document: Arc::new(Mutex::new(None)),
            shutting_down,
        }
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    pub fn mark_shutting_down(&self) -> bool {
        !self.shutting_down.swap(true, Ordering::SeqCst)
    }
}
