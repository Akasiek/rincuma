use jiff::Timestamp;
use serde::Serialize;

use crate::db::Project;

#[derive(Serialize)]
pub(super) struct ProjectResponse {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) color: Option<String>,
    pub(super) archived_at: Option<Timestamp>,
    pub(super) owner_id: i64,
    pub(super) created_at: Timestamp,
    pub(super) updated_at: Timestamp,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            color: project.color,
            archived_at: project.archived_at,
            owner_id: project.owner_id,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}
