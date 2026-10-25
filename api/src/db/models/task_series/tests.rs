#![allow(clippy::panic_in_result_fn)]

use super::{SeriesEnd, SeriesFrequency, TaskSeries};
use crate::{db::Task, recurrence::Schedule, web::test_support::state_with_user};
use jiff::{Timestamp, civil::date};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn create_series(
    executor: &mut dyn toasty::Executor,
    owner_id: i64,
    frequency: SeriesFrequency,
    end: SeriesEnd,
) -> Result<TaskSeries, Box<dyn std::error::Error>> {
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    Ok(toasty::create!(TaskSeries {
        owner_id,
        frequency,
        interval: 1,
        time_zone: "Europe/Warsaw".to_owned(),
        anchor: date(2028, 1, 31).at(9, 0, 0, 0),
        end,
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?)
}

async fn create_task(
    executor: &mut dyn toasty::Executor,
    owner_id: i64,
    series_id: Option<i64>,
) -> Result<Task, Box<dyn std::error::Error>> {
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    Ok(toasty::create!(Task {
        owner_id,
        name: "Recurring task".to_owned(),
        due_at: Some(now),
        series_id,
        scheduled_at: series_id.map(|_| now),
        created_at: now,
        updated_at: now,
    })
    .exec(executor)
    .await?)
}

#[tokio::test]
async fn round_trips_frequencies_and_mutually_exclusive_end_modes() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();

    for frequency in [
        SeriesFrequency::Daily,
        SeriesFrequency::Weekly,
        SeriesFrequency::Monthly,
    ] {
        for end in [
            SeriesEnd::Never,
            SeriesEnd::Through {
                date: date(2028, 12, 31),
            },
            SeriesEnd::After { count: 12 },
        ] {
            let series = create_series(&mut db, user.id, frequency, end).await?;
            let stored = TaskSeries::get_by_id(&mut db, series.id).await?;
            assert_eq!(stored.owner_id, user.id);
            assert_eq!(stored.frequency, frequency);
            assert_eq!(stored.interval, 1);
            assert_eq!(stored.time_zone, "Europe/Warsaw");
            assert_eq!(stored.anchor, date(2028, 1, 31).at(9, 0, 0, 0));
            assert_eq!(stored.end, end);
            assert_eq!(stored.occurrence_count, 1);
            assert_eq!(stored.latest_task_id, None);
            assert_eq!(stored.stopped_at, None);
            assert_eq!(stored.created_at, series.created_at);
            assert_eq!(stored.updated_at, series.updated_at);
        }
    }
    Ok(())
}

#[tokio::test]
async fn persisted_rule_preserves_month_end_intent() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let series =
        create_series(&mut db, user.id, SeriesFrequency::Monthly, SeriesEnd::Never).await?;
    let stored = TaskSeries::get_by_id(&mut db, series.id).await?;
    let schedule = Schedule::try_from(&stored)?;

    let january: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let february: Timestamp = "2028-02-29T08:00:00Z".parse()?;
    assert_eq!(schedule.next_after(january, january, 1)?, Some(february));
    assert_eq!(
        schedule.next_after(february, february, 2)?,
        Some("2028-03-31T07:00:00Z".parse()?),
    );
    Ok(())
}

#[tokio::test]
async fn ordinary_tasks_have_no_recurrence_metadata() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let task = create_task(&mut db, user.id, None).await?;
    let stored = Task::get_by_id(&mut db, task.id).await?;

    assert_eq!(stored.series_id, None);
    assert_eq!(stored.scheduled_at, None);
    assert_eq!(stored.recurrence_modified_at, None);
    Ok(())
}

#[tokio::test]
async fn occurrence_due_date_can_change_without_losing_the_original_slot() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let series = create_series(&mut db, user.id, SeriesFrequency::Daily, SeriesEnd::Never).await?;
    let mut task = create_task(&mut db, user.id, Some(series.id)).await?;
    let original_slot = task.scheduled_at;
    let changed_due: Timestamp = "2028-02-05T10:00:00Z".parse()?;

    // These edits share the same audit timestamp; undo must use durable metadata.
    toasty::update!(task {
        due_at: Some(changed_due),
        recurrence_modified_at: Some(task.updated_at),
    })
    .exec(&mut db)
    .await?;

    let stored = Task::get_by_id(&mut db, task.id).await?;
    assert_eq!(stored.series_id, Some(series.id));
    assert_eq!(stored.due_at, Some(changed_due));
    assert_eq!(stored.scheduled_at, original_slot);
    assert_eq!(stored.updated_at, stored.created_at);
    assert_eq!(stored.recurrence_modified_at, Some(stored.updated_at));
    Ok(())
}

#[tokio::test]
async fn detaching_an_occurrence_keeps_its_series_and_completed_history() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let mut series =
        create_series(&mut db, user.id, SeriesFrequency::Daily, SeriesEnd::Never).await?;
    let mut historical = create_task(&mut db, user.id, Some(series.id)).await?;
    let mut current = create_task(&mut db, user.id, Some(series.id)).await?;
    let now: Timestamp = "2028-02-01T08:00:00Z".parse()?;

    let mut transaction = db.transaction().await?;
    toasty::update!(historical {
        completed_at: Some(now)
    })
    .exec(&mut transaction)
    .await?;
    toasty::update!(current {
        series_id: None,
        scheduled_at: None,
    })
    .exec(&mut transaction)
    .await?;
    toasty::update!(series {
        occurrence_count: 2,
        latest_task_id: None,
        stopped_at: Some(now),
    })
    .exec(&mut transaction)
    .await?;
    transaction.commit().await?;

    let historical = Task::get_by_id(&mut db, historical.id).await?;
    let current = Task::get_by_id(&mut db, current.id).await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(historical.series_id, Some(series.id));
    assert_eq!(historical.completed_at, Some(now));
    assert_eq!(current.series_id, None);
    assert_eq!(current.scheduled_at, None);
    assert_eq!(series.stopped_at, Some(now));
    assert_eq!(series.occurrence_count, 2);
    Ok(())
}

#[tokio::test]
async fn rollback_preserves_the_count_cursor_and_occurrences_together() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let mut series = create_series(
        &mut db,
        user.id,
        SeriesFrequency::Daily,
        SeriesEnd::After { count: 2 },
    )
    .await?;
    let original = create_task(&mut db, user.id, Some(series.id)).await?;
    toasty::update!(series {
        latest_task_id: Some(original.id)
    })
    .exec(&mut db)
    .await?;

    let mut transaction = db.transaction().await?;
    let successor = create_task(&mut transaction, user.id, Some(series.id)).await?;
    toasty::update!(series {
        latest_task_id: Some(successor.id),
        occurrence_count: 2,
    })
    .exec(&mut transaction)
    .await?;
    transaction.rollback().await?;

    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let tasks = Task::filter(Task::fields().series_id().eq(Some(series.id)))
        .exec(&mut db)
        .await?;
    assert_eq!(series.occurrence_count, 1);
    assert_eq!(series.latest_task_id, Some(original.id));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks.first().map(|task| task.id), Some(original.id));
    Ok(())
}
