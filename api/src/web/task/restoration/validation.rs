use std::collections::{BTreeSet, HashSet};

use jiff::Timestamp;

use crate::{
    db::{Task, TaskSeries},
    web::task::TaskError,
};

const CHANGED: &str = "The task's parent or recurring series changed after deletion.";

/// Uses the same series-before-tasks lock order as edits, completion, and deletion.
pub(super) async fn lock_deleted_tasks(
    executor: &mut dyn toasty::Executor,
    root_id: i64,
    owner_id: i64,
    deleted_at: Timestamp,
) -> Result<Vec<Task>, TaskError> {
    let selected = Task::filter(Task::fields().deletion_root_id().eq(Some(root_id)))
        .filter(Task::fields().deleted_at().eq(Some(deleted_at)))
        .filter(Task::fields().owner_id().eq(owner_id))
        .order_by(Task::fields().id().asc())
        .exec(executor)
        .await?;

    let series_ids: BTreeSet<_> = selected.iter().filter_map(|task| task.series_id).collect();
    for id in series_ids {
        TaskSeries::lock_owned(executor, id, owner_id)
            .await?
            .ok_or_else(|| TaskError::conflict(CHANGED))?;
    }

    let mut tasks = Vec::new();
    for task in selected {
        let locked = Task::lock_owned_including_deleted(executor, task.id, owner_id)
            .await?
            .filter(|current| current.deleted_at.is_some())
            .ok_or_else(TaskError::related_resource_not_found)?;

        if locked.deleted_at != Some(deleted_at)
            || locked.deletion_root_id != Some(root_id)
            || locked.series_id != task.series_id
        {
            return Err(TaskError::conflict(CHANGED));
        }
        tasks.push(locked);
    }

    Ok(tasks)
}

pub(super) async fn validate_restoration(
    executor: &mut dyn toasty::Executor,
    tasks: &[Task],
    deleted_at: Timestamp,
) -> Result<(), TaskError> {
    if tasks.is_empty() {
        return Err(TaskError::related_resource_not_found());
    }

    let ids: HashSet<_> = tasks.iter().map(|task| task.id).collect();
    for task in tasks {
        validate_parent(executor, task, &ids).await?;
        validate_series(executor, task, deleted_at).await?;
    }

    Ok(())
}

async fn validate_parent(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    restored_ids: &HashSet<i64>,
) -> Result<(), TaskError> {
    let Some(parent_id) = task.parent_id else {
        return Ok(());
    };
    if restored_ids.contains(&parent_id) {
        return Ok(());
    }

    let parent = Task::lock_owned(executor, parent_id, task.owner_id)
        .await?
        .ok_or_else(|| TaskError::conflict(CHANGED))?;
    if parent.project_id != task.project_id || parent.series_id.is_some() {
        return Err(TaskError::conflict(CHANGED));
    }

    Ok(())
}

async fn validate_series(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    deleted_at: Timestamp,
) -> Result<(), TaskError> {
    let Some(id) = task.stopped_series_id else {
        return Ok(());
    };
    if task.series_id != Some(id) {
        return Err(TaskError::conflict(CHANGED));
    }

    let series = TaskSeries::lock_owned(executor, id, task.owner_id)
        .await?
        .ok_or_else(|| TaskError::conflict(CHANGED))?;
    if series.stopped_at != Some(deleted_at)
        || series.updated_at != deleted_at
        || series.latest_task_id != Some(task.id)
    {
        return Err(TaskError::conflict(CHANGED));
    }

    Ok(())
}
