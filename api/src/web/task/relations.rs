use crate::{
    db::{Project, Tag},
    web::task::TaskError,
};

pub(super) async fn validate_active_project(
    executor: &mut dyn toasty::Executor,
    project_id: i64,
    owner_id: i64,
) -> Result<(), TaskError> {
    Project::get_owned(executor, project_id, owner_id)
        .await?
        .filter(|project| project.archived_at.is_none())
        .ok_or_else(TaskError::related_resource_not_found)
        .map(|_| ())
}

pub(super) async fn validate_owned_tag(
    executor: &mut dyn toasty::Executor,
    tag_id: i64,
    owner_id: i64,
) -> Result<(), TaskError> {
    Tag::get_owned(executor, tag_id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)
        .map(|_| ())
}
