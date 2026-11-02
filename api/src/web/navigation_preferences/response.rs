use crate::db::NavigationPreferences;
use serde::Serialize;

#[derive(Default, Serialize)]
pub(super) struct PreferencesResponse {
    pub(super) projects_open: Option<bool>,
    pub(super) tags_open: Option<bool>,
}

impl From<NavigationPreferences> for PreferencesResponse {
    fn from(value: NavigationPreferences) -> Self {
        Self {
            projects_open: value.projects_open,
            tags_open: value.tags_open,
        }
    }
}
