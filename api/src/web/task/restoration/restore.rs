use jiff::Timestamp;

use super::validation::{lock_deleted_tasks, validate_restoration};
use crate::{
    db::{Task, TaskSeries},
    web::task::{TaskError, deletion::undo_expires_at},
};

/// Unhides one root/timestamp group atomically, preserving the series-before-tasks
/// lock order. Concurrent restores recheck the root after acquiring those locks.
pub(in crate::web::task) async fn restore_deletion(
    executor: &mut dyn toasty::Executor,
    id: i64,
    owner_id: i64,
    deleted_at: Timestamp,
    now: Timestamp,
) -> Result<(), TaskError> {
    validate_deleted_root(executor, id, owner_id, deleted_at).await?;
    if undo_expires_at(deleted_at)? <= now {
        return Err(TaskError::conflict("The deletion can no longer be undone."));
    }
    
    let tasks = lock_deleted_tasks(executor, id, owner_id, deleted_at).await?;
    validate_restoration(executor, &tasks, deleted_at).await?;
    resume_series(executor, &tasks, now).await?;
    
    for task in tasks {
        toasty::update!(Task::filter_by_id(task.id) {
            deleted_at: None,
            deletion_root_id: None,
            stopped_series_id: None,
        })
        .exec(executor)
        .await?;
    }
    
    Ok(())
}

async fn validate_deleted_root(
    executor: &mut dyn toasty::Executor,
    id: i64,
    owner_id: i64,
    deleted_at: Timestamp,
) -> Result<(), TaskError> {
    let root = Task::filter_by_id(id)
        .filter(Task::fields().owner_id().eq(owner_id))
        .first()
        .exec(executor)
        .await?
        .filter(|task| task.deleted_at.is_some())
        .ok_or_else(TaskError::related_resource_not_found)?;

    if root.deleted_at != Some(deleted_at) || root.deletion_root_id != Some(id) {
        return Err(TaskError::conflict("This task's deletion has changed."));
    }

    Ok(())
}

async fn resume_series(
    executor: &mut dyn toasty::Executor,
    tasks: &[Task],
    now: Timestamp,
) -> Result<(), TaskError> {
    for id in tasks.iter().filter_map(|task| task.stopped_series_id) {
        toasty::update!(TaskSeries::filter_by_id(id) {
            stopped_at: None,
            updated_at: now
        })
        .exec(executor)
        .await?;
    }

    Ok(())
}
