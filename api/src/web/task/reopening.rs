use jiff::Timestamp;

use crate::{
    db::{Task, TaskSeries},
    web::task::TaskError,
};

/// Reopens a locked occurrence and removes its untouched successor atomically.
/// The caller must lock the series before any task, using `lock_owned_task`.
pub(super) async fn reopen_locked(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    now: Timestamp,
) -> Result<(), TaskError> {
    if task.completed_at.is_none() {
        return Ok(());
    }
    if let Some(series_id) = task.series_id {
        undo_successor(executor, task, series_id, now).await?;
    }

    toasty::update!(task {
        completed_at: None,
        updated_at: now
    })
    .exec(executor)
    .await?;
    Ok(())
}

async fn undo_successor(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    series_id: i64,
    now: Timestamp,
) -> Result<(), TaskError> {
    let mut series = TaskSeries::lock_owned(executor, series_id, task.owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    if series.stopped_at.is_some() {
        return Err(TaskError::conflict(
            "This task's recurring series has been stopped.",
        ));
    }

    validate_latest_completed(executor, task, series_id).await?;
    let latest_id = series.latest_task_id.ok_or_else(|| {
        TaskError::conflict("The next occurrence is no longer part of this series.")
    })?;

    let mut count = series.occurrence_count;
    if latest_id != task.id {
        let successor = Task::lock_owned(executor, latest_id, task.owner_id)
            .await?
            .ok_or_else(|| TaskError::conflict("The next occurrence is no longer available."))?;
        validate_successor(executor, task, &successor).await?;
        count = count
            .checked_sub(1)
            .filter(|count| *count > 0)
            .ok_or_else(TaskError::bad_request)?;
        // Undo cancels this occurrence's creation, so remove the row entirely.
        // User-requested task deletion uses soft deletion instead.
        Task::filter_by_id(successor.id)
            .delete()
            .exec(executor)
            .await?;
    }

    toasty::update!(series {
        latest_task_id: Some(task.id),
        occurrence_count: count,
        updated_at: now,
    })
    .exec(executor)
    .await?;
    Ok(())
}

async fn validate_latest_completed(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    series_id: i64,
) -> Result<(), TaskError> {
    // Occurrences are created under the series lock with fresh auto-increment IDs.
    // Their creation order remains stable even if due dates or completion times change.
    let latest = Task::not_deleted()
        .filter(Task::fields().series_id().eq(Some(series_id)))
        .filter(Task::fields().completed_at().is_some())
        .order_by(Task::fields().id().desc())
        .first()
        .exec(executor)
        .await?;
    if latest.as_ref().map(|task| task.id) != Some(task.id) {
        return Err(TaskError::conflict(
            "Only the latest completed occurrence can be reopened.",
        ));
    }
    Ok(())
}

async fn validate_successor(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    successor: &Task,
) -> Result<(), TaskError> {
    if successor.series_id != task.series_id || successor.predecessor_task_id != Some(task.id) {
        return Err(TaskError::conflict(
            "The next occurrence is no longer part of this series.",
        ));
    }

    if successor.completed_at.is_some() || successor.recurrence_modified_at.is_some() {
        return Err(TaskError::conflict(
            "The next occurrence has been changed or completed.",
        ));
    }

    let child = Task::not_deleted()
        .filter(Task::fields().parent_id().eq(Some(successor.id)))
        .first()
        .exec(executor)
        .await?;
    if child.is_some() {
        return Err(TaskError::conflict(
            "The next occurrence has subtasks and cannot be removed.",
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/recurring_reopening.rs"]
mod tests;
