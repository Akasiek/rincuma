use crate::db::NotificationSettings;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct SettingsResponse {
    pub(super) provider: String,
    pub(super) enabled: bool,
    pub(super) url: String,
    pub(super) has_authorization: bool,
}

impl From<NotificationSettings> for SettingsResponse {
    fn from(value: NotificationSettings) -> Self {
        Self {
            provider: value.provider,
            enabled: value.enabled,
            url: value.url,
            has_authorization: value.authorization.is_some(),
        }
    }
}
