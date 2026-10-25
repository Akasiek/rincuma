use jiff::{Timestamp, tz::TimeZone};

use super::{RecurrenceChange, RecurrenceConfig};
use crate::{
    db::{SeriesEnd, Task, TaskSeries},
    recurrence::Schedule,
    web::task::{TaskError, relations::validate_recurrence_leaf},
};

/// Applies configuration inside the task save transaction, under its series/task locks.
pub(in crate::web::task) async fn apply_recurrence(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    change: RecurrenceChange,
    now: Timestamp,
) -> Result<(), TaskError> {
    match change {
        RecurrenceChange::Keep => keep(task),
        RecurrenceChange::Disable => disable(executor, task, now).await,
        RecurrenceChange::Set(config) => set(executor, task, config, now).await,
    }
}

fn keep(task: &Task) -> Result<(), TaskError> {
    if task.series_id.is_some() && task.due_at.is_none() {
        return Err(TaskError::conflict(
            "Disable recurrence before removing the due date.",
        ));
    }
    Ok(())
}

async fn disable(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    now: Timestamp,
) -> Result<(), TaskError> {
    let Some(series_id) = task.series_id else {
        return Ok(());
    };

    let mut series = owned_active_series(executor, task, series_id).await?;
    toasty::update!(series {
        stopped_at: Some(now),
        updated_at: now
    })
    .exec(executor)
    .await?;

    toasty::update!(task {
        series_id: None,
        scheduled_at: None,
        predecessor_task_id: None,
        recurrence_modified_at: Some(now),
    })
    .exec(executor)
    .await?;

    Ok(())
}

async fn set(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    mut config: RecurrenceConfig,
    now: Timestamp,
) -> Result<(), TaskError> {
    validate_recurrence_leaf(executor, task.id).await?;
    let due_at = task.due_at.ok_or_else(TaskError::bad_request)?;
    normalize_config(&mut config, due_at)?;

    if let Some(series_id) = task.series_id {
        change_rule(executor, task, series_id, &config, due_at, now).await
    } else {
        enable(executor, task, &config, due_at, now).await
    }
}

fn normalize_config(config: &mut RecurrenceConfig, due_at: Timestamp) -> Result<(), TaskError> {
    let zone = TimeZone::get(&config.time_zone).map_err(|_| TaskError::bad_request())?;
    let canonical_name = zone.iana_name().ok_or_else(TaskError::bad_request)?;
    canonical_name.clone_into(&mut config.time_zone);

    Schedule::new(
        config.frequency.into(),
        config.interval,
        &config.time_zone,
        due_at.to_zoned(zone).datetime(),
        config.end.into(),
    )
    .map_err(|_| TaskError::bad_request())?;

    Ok(())
}

async fn enable(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    config: &RecurrenceConfig,
    due_at: Timestamp,
    now: Timestamp,
) -> Result<(), TaskError> {
    if task.completed_at.is_some() {
        return Err(TaskError::conflict(
            "Reopen this task before enabling recurrence.",
        ));
    }

    let anchor = due_at
        .in_tz(&config.time_zone)
        .map_err(|_| TaskError::bad_request())?
        .datetime();
    let series = toasty::create!(TaskSeries {
        owner_id: task.owner_id,
        frequency: config.frequency,
        interval: config.interval,
        time_zone: config.time_zone.clone(),
        anchor,
        end: config.end,
        latest_task_id: Some(task.id),
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?;

    toasty::update!(task {
        series_id: Some(series.id),
        scheduled_at: Some(due_at),
        predecessor_task_id: None,
        recurrence_modified_at: None,
    })
    .exec(executor)
    .await?;

    Ok(())
}

async fn change_rule(
    executor: &mut dyn toasty::Executor,
    task: &mut Task,
    series_id: i64,
    config: &RecurrenceConfig,
    due_at: Timestamp,
    now: Timestamp,
) -> Result<(), TaskError> {
    let mut series = owned_active_series(executor, task, series_id).await?;
    if let SeriesEnd::After { count } = config.end
        && count < series.occurrence_count
    {
        return Err(TaskError::bad_request());
    }

    let unchanged = series.frequency == config.frequency
        && series.interval == config.interval
        && series.time_zone == config.time_zone
        && series.end == config.end;
    if unchanged {
        return Ok(());
    }

    let anchor = due_at
        .in_tz(&config.time_zone)
        .map_err(|_| TaskError::bad_request())?
        .datetime();
    toasty::update!(series {
        frequency: config.frequency,
        interval: config.interval,
        time_zone: config.time_zone.clone(),
        anchor,
        end: config.end,
        updated_at: now,
    })
    .exec(executor)
    .await?;

    toasty::update!(task {
        scheduled_at: Some(due_at),
        recurrence_modified_at: Some(now)
    })
    .exec(executor)
    .await?;

    Ok(())
}

async fn owned_active_series(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    series_id: i64,
) -> Result<TaskSeries, TaskError> {
    let series = TaskSeries::lock_owned(executor, series_id, task.owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;

    if task.completed_at.is_some()
        || series.stopped_at.is_some()
        || series.latest_task_id != Some(task.id)
    {
        return Err(TaskError::conflict(
            "Only the active occurrence's recurrence can be changed.",
        ));
    }

    Ok(series)
}

#[cfg(test)]
#[path = "../tests/recurrence_configuration.rs"]
mod tests;
