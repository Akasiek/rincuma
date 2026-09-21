use crate::config::AppConfig;
use std::sync::Arc;
use toasty::Db;

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    config: Arc<AppConfig>,
    db: Db,
}

impl AppState {
    pub(crate) fn new(config: AppConfig, db: Db) -> Self {
        Self {
            config: Arc::new(config),
            db,
        }
    }

    pub(crate) fn config(&self) -> &AppConfig {
        &self.config
    }

    pub(crate) fn db(&self) -> &Db {
        &self.db
    }
}
