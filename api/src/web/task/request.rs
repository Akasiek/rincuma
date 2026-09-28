use std::collections::HashSet;

use garde::Validate;
use jiff::Timestamp;
use serde::Deserialize;

use crate::db::TaskPriority;

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub(super) struct SaveTaskRequest {
    #[garde(length(bytes, min = 1, max = 255))]
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) due_at: Option<Timestamp>,
    pub(super) priority: Option<TaskPriority>,
    pub(super) project_id: Option<i64>,
    pub(super) parent_id: Option<i64>,
    #[serde(default)]
    #[garde(custom(validate_tag_ids))]
    pub(super) tag_ids: Vec<i64>,
}

fn validate_tag_ids(tag_ids: &[i64], _: &()) -> garde::Result {
    let mut seen = HashSet::new();
    if tag_ids.iter().any(|&id| id <= 0 || !seen.insert(id)) {
        Err(garde::Error::new("tag IDs must be positive and unique"))
    } else {
        Ok(())
    }
}
