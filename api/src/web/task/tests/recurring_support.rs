use jiff::{Timestamp, civil::date};

use crate::{
    app_state::AppState,
    db::{SeriesEnd, SeriesFrequency, Task, TaskPriority, TaskSeries, TaskTag},
    web::task::{
        TaskError, completion::complete_locked, locking::lock_owned_task, reopening::reopen_locked,
    },
};

pub(in crate::web::task) async fn recurring(
    executor: &mut dyn toasty::Executor,
    owner_id: i64,
    end: SeriesEnd,
) -> Result<(Task, TaskSeries), Box<dyn std::error::Error>> {
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let mut series = toasty::create!(TaskSeries {
        owner_id,
        frequency: SeriesFrequency::Monthly,
        interval: 1,
        time_zone: "Europe/Warsaw".to_owned(),
        anchor: date(2028, 1, 31).at(9, 0, 0, 0),
        end,
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?;
    let task = toasty::create!(Task {
        owner_id,
        name: "Original task".to_owned(),
        description: Some("Original description".to_owned()),
        priority: TaskPriority::High,
        due_at: Some(now),
        series_id: Some(series.id),
        scheduled_at: Some(now),
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?;
    toasty::update!(series {
        latest_task_id: Some(task.id)
    })
    .exec(executor)
    .await?;
    Ok((task, series))
}

pub(in crate::web::task) async fn finish_at(
    state: &AppState,
    owner_id: i64,
    id: i64,
    now: &str,
) -> Result<Task, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let mut task = lock_owned_task(&mut transaction, id, owner_id).await?;
    let tags = TaskTag::filter_by_task_id(id)
        .exec(&mut transaction)
        .await?;
    let tag_ids: Vec<i64> = tags.into_iter().map(|tag| tag.tag_id).collect();
    let now = now.parse().map_err(|_| TaskError::bad_request())?;
    complete_locked(&mut transaction, &mut task, &tag_ids, now).await?;
    transaction.commit().await?;
    Ok(task)
}

pub(in crate::web::task) async fn occurrences(
    executor: &mut dyn toasty::Executor,
    id: i64,
) -> toasty::Result<Vec<Task>> {
    Task::filter(Task::fields().series_id().eq(Some(id)))
        .order_by(Task::fields().id().asc())
        .exec(executor)
        .await
}

pub(in crate::web::task) async fn reopen_at(
    state: &AppState,
    owner_id: i64,
    id: i64,
    now: &str,
) -> Result<Task, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let mut task = lock_owned_task(&mut transaction, id, owner_id).await?;
    let now = now.parse().map_err(|_| TaskError::bad_request())?;
    reopen_locked(&mut transaction, &mut task, now).await?;
    transaction.commit().await?;
    Ok(task)
}
