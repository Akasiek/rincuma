use serde::Serialize;

use crate::config::AppConfig;

/// Instance-wide settings that are safe to expose without authentication.
#[derive(Serialize)]
pub(super) struct InstanceResponse {
    pub(super) registration_enabled: bool,
}

impl From<&AppConfig> for InstanceResponse {
    fn from(config: &AppConfig) -> Self {
        Self {
            registration_enabled: config.registration_enabled(),
        }
    }
}
