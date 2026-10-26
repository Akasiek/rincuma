use jiff::Timestamp;
use serde::Serialize;

use crate::db::Tag;

#[derive(Serialize)]
pub(super) struct TagResponse {
    pub(super) id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) task_count: Option<usize>,
    pub(super) name: String,
    pub(super) color: Option<String>,
    pub(super) owner_id: i64,
    pub(super) created_at: Timestamp,
    pub(super) updated_at: Timestamp,
}

impl TagResponse {
    pub(super) async fn for_list(
        db: &mut dyn toasty::Executor,
        owner_id: i64,
        tags: Vec<Tag>,
    ) -> toasty::Result<Vec<Self>> {
        let ids: Vec<_> = tags.iter().map(|tag| tag.id).collect();
        let counts = crate::web::dictionary_counts::tags(db, owner_id, &ids).await?;

        Ok(tags
            .into_iter()
            .map(|tag| {
                let task_count = counts.get(&tag.id).copied().unwrap_or_default();
                Self {
                    task_count: Some(task_count),
                    ..tag.into()
                }
            })
            .collect())
    }
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            task_count: None,
            name: tag.name,
            color: tag.color,
            owner_id: tag.owner_id,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}
