use crate::{
    db::{Project, Tag, Task},
    web::task::error::TaskError,
};

pub(super) async fn get_owned_task(
    executor: &mut dyn toasty::Executor,
    task_id: i64,
    owner_id: i64,
) -> Result<Task, TaskError> {
    Task::filter_by_id(task_id)
        .first()
        .exec(executor)
        .await?
        .filter(|task| task.owner_id == owner_id)
        .ok_or(TaskError::RelatedResourceNotFound)
}

pub(super) async fn validate_active_project(
    executor: &mut dyn toasty::Executor,
    project_id: i64,
    owner_id: i64,
) -> Result<(), TaskError> {
    Project::filter_by_id(project_id)
        .first()
        .exec(executor)
        .await?
        .filter(|project| project.owner_id == owner_id && project.archived_at.is_none())
        .ok_or(TaskError::RelatedResourceNotFound)
        .map(|_| ())
}

pub(super) async fn validate_owned_tag(
    executor: &mut dyn toasty::Executor,
    tag_id: i64,
    owner_id: i64,
) -> Result<(), TaskError> {
    Tag::filter_by_id(tag_id)
        .first()
        .exec(executor)
        .await?
        .filter(|tag| tag.owner_id == owner_id)
        .ok_or(TaskError::RelatedResourceNotFound)
        .map(|_| ())
}
