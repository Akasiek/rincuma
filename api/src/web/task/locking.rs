use crate::{
    db::{Task, TaskSeries},
    web::task::TaskError,
};

/// Locks a series before its tasks so completion, edits, and undo share one order.
/// Rechecks membership after waiting; an occurrence may have been detached or deleted.
pub(super) async fn lock_owned_task(
    executor: &mut dyn toasty::Executor,
    id: i64,
    owner_id: i64,
) -> Result<Task, TaskError> {
    let selected = Task::get_owned(executor, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    if let Some(series_id) = selected.series_id {
        TaskSeries::lock_owned(executor, series_id, owner_id)
            .await?
            .ok_or_else(TaskError::related_resource_not_found)?;
    }

    let task = Task::lock_owned(executor, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    if task.series_id != selected.series_id {
        return Err(TaskError::conflict(
            "This task's recurrence changed. Please try again.",
        ));
    }

    Ok(task)
}
