use jiff::Timestamp;
use serde::Serialize;

use crate::db::Project;

#[derive(Serialize)]
pub(super) struct ProjectResponse {
    pub(super) id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) task_count: Option<usize>,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) color: Option<String>,
    pub(super) archived_at: Option<Timestamp>,
    pub(super) owner_id: i64,
    pub(super) created_at: Timestamp,
    pub(super) updated_at: Timestamp,
}

impl ProjectResponse {
    pub(super) async fn for_list(
        db: &mut dyn toasty::Executor,
        owner_id: i64,
        projects: Vec<Project>,
    ) -> toasty::Result<Vec<Self>> {
        let ids: Vec<_> = projects.iter().map(|project| project.id).collect();
        let counts = crate::web::dictionary_counts::projects(db, owner_id, &ids).await?;
        
        Ok(projects
            .into_iter()
            .map(|project| {
                let task_count = counts.get(&project.id).copied().unwrap_or_default();
                Self {
                    task_count: Some(task_count),
                    ..project.into()
                }
            })
            .collect())
    }
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            task_count: None,
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
