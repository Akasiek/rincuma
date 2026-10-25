use jiff::Timestamp;

use crate::{
    db::{Project, Task, TaskSeries, TaskTag},
    recurrence::Schedule,
    web::task::{
        TaskError,
        relations::{validate_owned_tag, validate_recurrence_leaf},
    },
};

/// Completes an already locked task and creates its successor in the same transaction.
pub(super) async fn complete_locked(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    tag_ids: &[i64],
    now: Timestamp,
) -> Result<(), TaskError> {
    if task.completed_at.is_some() {
        return Ok(());
    }

    if let Some(series_id) = task.series_id {
        advance_series(executor, task, tag_ids, series_id, now).await?;
    }

    toasty::update!(task {
        completed_at: Some(now),
        updated_at: now,
    })
    .exec(executor)
    .await?;

    Ok(())
}

async fn advance_series(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    tag_ids: &[i64],
    series_id: i64,
    now: Timestamp,
) -> Result<(), TaskError> {
    let mut series = TaskSeries::lock_owned(executor, series_id, task.owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    if series.stopped_at.is_some() {
        return Ok(());
    }
    if series.latest_task_id != Some(task.id) || task.due_at.is_none() {
        return Err(TaskError::bad_request());
    }

    let scheduled_at = task.scheduled_at.ok_or_else(TaskError::bad_request)?;
    validate_recurrence_leaf(executor, task.id).await?;

    let schedule = Schedule::try_from(&series).map_err(|_| TaskError::bad_request())?;
    let next = schedule
        .next_after(scheduled_at, now, series.occurrence_count)
        .map_err(|_| TaskError::bad_request())?;

    let Some(next) = next else {
        return Ok(());
    };

    let count = series
        .occurrence_count
        .checked_add(1)
        .ok_or_else(TaskError::bad_request)?;

    validate_copied_relations(executor, task, tag_ids).await?;
    let successor = create_successor(executor, task, tag_ids, next, now).await?;
    toasty::update!(series {
        latest_task_id: Some(successor.id),
        occurrence_count: count,
        updated_at: now,
    })
    .exec(executor)
    .await?;

    Ok(())
}

async fn validate_copied_relations(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    tag_ids: &[i64],
) -> Result<(), TaskError> {
    if let Some(project_id) = task.project_id {
        Project::get_owned(executor, project_id, task.owner_id)
            .await?
            .ok_or_else(TaskError::related_resource_not_found)?;
    }

    if let Some(parent_id) = task.parent_id {
        let parent = Task::get_owned(executor, parent_id, task.owner_id)
            .await?
            .ok_or_else(TaskError::related_resource_not_found)?;
        if parent.project_id != task.project_id {
            return Err(TaskError::bad_request());
        }
    }

    for tag_id in tag_ids {
        validate_owned_tag(executor, *tag_id, task.owner_id).await?;
    }

    Ok(())
}

async fn create_successor(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    tag_ids: &[i64],
    scheduled_at: Timestamp,
    now: Timestamp,
) -> Result<Task, TaskError> {
    let successor = toasty::create!(Task {
        owner_id: task.owner_id,
        name: task.name.clone(),
        description: task.description.clone(),
        priority: task.priority,
        project_id: task.project_id,
        parent_id: task.parent_id,
        series_id: task.series_id,
        predecessor_task_id: Some(task.id),
        due_at: Some(scheduled_at),
        scheduled_at: Some(scheduled_at),
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?;

    for tag_id in tag_ids {
        toasty::create!(TaskTag {
            task_id: successor.id,
            tag_id: *tag_id
        })
        .exec(executor)
        .await?;
    }

    Ok(successor)
}

#[cfg(test)]
#[path = "tests/recurring_completion.rs"]
mod tests;
