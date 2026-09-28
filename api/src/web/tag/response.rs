use jiff::Timestamp;
use serde::Serialize;

use crate::db::Tag;

#[derive(Serialize)]
pub(super) struct TagResponse {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) color: Option<String>,
    pub(super) owner_id: i64,
    pub(super) created_at: Timestamp,
    pub(super) updated_at: Timestamp,
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            color: tag.color,
            owner_id: tag.owner_id,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}
