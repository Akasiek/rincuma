#![allow(clippy::panic_in_result_fn)]

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{Request, StatusCode},
};
use jiff::{Timestamp, civil::date};
use tower::ServiceExt;

use crate::{
    app_state::AppState,
    auth::session::create_session,
    db::{SeriesEnd, SeriesFrequency, Task, TaskSeries, User},
    web::{
        auth::CurrentUser,
        router::get_app_router,
        task::{
            TaskError,
            handlers::{ListTasksQuery, create, get_task, list, update},
            recurrence::{RecurrenceChange, RecurrenceConfig},
            recurring_support::{finish_at, recurring, reopen_at},
            request::SaveTaskRequest,
        },
        test_support::state_with_user,
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn config(end: SeriesEnd) -> RecurrenceConfig {
    RecurrenceConfig {
        frequency: SeriesFrequency::Monthly,
        interval: 1,
        time_zone: "Europe/Warsaw".to_owned(),
        end,
    }
}

fn request() -> Result<SaveTaskRequest, Box<dyn std::error::Error>> {
    Ok(SaveTaskRequest {
        name: "Recurring task".to_owned(),
        description: None,
        due_at: Some("2028-01-31T08:00:00Z".parse()?),
        priority: None,
        parent_id: None,
        project_id: None,
        tag_ids: Vec::new(),
        recurrence: RecurrenceChange::Set(config(SeriesEnd::Never)),
    })
}

fn edit(task: &Task, change: RecurrenceChange) -> SaveTaskRequest {
    SaveTaskRequest {
        name: task.name.clone(),
        description: task.description.clone(),
        due_at: task.due_at,
        priority: Some(task.priority),
        parent_id: task.parent_id,
        project_id: task.project_id,
        tag_ids: Vec::new(),
        recurrence: change,
    }
}

async fn save(state: &AppState, task: &Task, request: SaveTaskRequest) -> Result<(), TaskError> {
    let mut db = state.db().clone();
    let user = User::get_by_id(&mut db, task.owner_id).await?;
    let _ = update(
        State(state.clone()),
        CurrentUser(user),
        Path(task.id),
        Json(request),
    )
    .await?;
    Ok(())
}

async fn successor(
    state: &AppState,
    owner_id: i64,
) -> Result<(Task, Task, TaskSeries), Box<dyn std::error::Error>> {
    let mut db = state.db().clone();
    let (first, series) = recurring(&mut db, owner_id, SeriesEnd::Never).await?;
    let first = finish_at(state, owner_id, first.id, "2028-01-31T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let current =
        Task::get_by_id(&mut db, series.latest_task_id.ok_or("missing successor")?).await?;
    Ok((first, current, series))
}

#[tokio::test]
async fn creates_task_and_series_atomically_and_returns_its_rule() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (status, Json(response)) =
        create(State(state.clone()), CurrentUser(user), Json(request()?)).await?;
    assert_eq!(status, StatusCode::CREATED);
    let task = Task::get_by_id(&mut db, response.id).await?;
    let series = TaskSeries::get_by_id(&mut db, task.series_id.ok_or("missing series")?).await?;
    assert_eq!(series.anchor, date(2028, 1, 31).at(9, 0, 0, 0));
    assert_eq!(series.latest_task_id, Some(task.id));
    assert_eq!(series.occurrence_count, 1);
    assert_eq!(task.scheduled_at, task.due_at);
    let response = response.recurrence.ok_or("missing rule response")?;
    assert_eq!(response.config, config(SeriesEnd::Never));
    assert_eq!(response.occurrence_count, 1);
    Ok(())
}

#[tokio::test]
async fn invalid_rules_roll_back_task_creation() -> TestResult {
    let mut rules = vec![config(SeriesEnd::After { count: 0 })];
    let mut zero_interval = config(SeriesEnd::Never);
    zero_interval.interval = 0;
    rules.push(zero_interval);
    for zone in ["Not/A_Zone", "+02:00", "Etc/Unknown"] {
        let mut invalid = config(SeriesEnd::Never);
        invalid.time_zone = zone.to_owned();
        rules.push(invalid);
    }
    for rule in rules {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let owner_id = user.id;
        let mut request = request()?;
        request.recurrence = RecurrenceChange::Set(rule);
        let error = create(State(state.clone()), CurrentUser(user), Json(request))
            .await
            .err()
            .ok_or("expected rule rejection")?;
        assert!(matches!(error, TaskError::BadRequest(_)));
        assert_eq!(
            Task::filter(Task::fields().owner_id().eq(owner_id))
                .count()
                .exec(&mut db)
                .await?,
            0
        );
        assert_eq!(
            TaskSeries::filter(TaskSeries::fields().owner_id().eq(owner_id))
                .count()
                .exec(&mut db)
                .await?,
            0
        );
    }
    Ok(())
}

#[tokio::test]
async fn enabling_requires_a_due_time_and_an_incomplete_leaf_task() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let mut task = toasty::create!(Task {
        owner_id: user.id,
        name: "Ordinary".to_owned(),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    assert!(
        save(
            &state,
            &task,
            edit(&task, RecurrenceChange::Set(config(SeriesEnd::Never)))
        )
        .await
        .is_err()
    );
    toasty::update!(task {
        due_at: Some(now),
        completed_at: Some(now)
    })
    .exec(&mut db)
    .await?;
    assert!(
        save(
            &state,
            &task,
            edit(&task, RecurrenceChange::Set(config(SeriesEnd::Never)))
        )
        .await
        .is_err()
    );
    toasty::update!(task { completed_at: None })
        .exec(&mut db)
        .await?;
    toasty::create!(Task {
        owner_id: user.id,
        name: "Child".to_owned(),
        parent_id: Some(task.id),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    assert!(
        save(
            &state,
            &task,
            edit(&task, RecurrenceChange::Set(config(SeriesEnd::Never)))
        )
        .await
        .is_err()
    );
    assert_eq!(Task::get_by_id(&mut db, task.id).await?.series_id, None);
    Ok(())
}

#[tokio::test]
async fn occurrence_only_rescheduling_preserves_the_original_calendar_anchor() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (_, task, series) = successor(&state, user.id).await?;
    let mut changed = edit(&task, RecurrenceChange::Keep);
    changed.due_at = Some("2028-03-05T08:00:00Z".parse()?);
    save(&state, &task, changed).await?;
    let stored = Task::get_by_id(&mut db, task.id).await?;
    assert_eq!(stored.scheduled_at, task.scheduled_at);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id).await?.anchor,
        series.anchor
    );
    let mut removed = edit(&stored, RecurrenceChange::Keep);
    removed.due_at = None;
    assert!(save(&state, &stored, removed).await.is_err());
    assert_eq!(
        Task::get_by_id(&mut db, task.id).await?.due_at,
        stored.due_at
    );
    Ok(())
}

#[tokio::test]
async fn editing_a_rule_reanchors_at_the_current_due_date_and_blocks_predecessor_undo() -> TestResult
{
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (historical, task, series) = successor(&state, user.id).await?;
    let mut rule = config(SeriesEnd::Never);
    rule.frequency = SeriesFrequency::Weekly;
    let mut changed = edit(&task, RecurrenceChange::Set(rule));
    changed.due_at = Some("2028-03-05T08:00:00Z".parse()?);
    save(&state, &task, changed).await?;
    let stored = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(stored.anchor, date(2028, 3, 5).at(9, 0, 0, 0));
    assert_eq!(stored.occurrence_count, 2);
    assert_eq!(stored.frequency, SeriesFrequency::Weekly);
    assert!(
        Task::get_by_id(&mut db, task.id)
            .await?
            .recurrence_modified_at
            .is_some()
    );
    assert!(
        reopen_at(&state, user.id, historical.id, "2028-03-05T08:05:00Z")
            .await
            .is_err()
    );
    assert_eq!(
        Task::get_by_id(&mut db, historical.id).await?.scheduled_at,
        historical.scheduled_at
    );
    finish_at(&state, user.id, task.id, "2028-03-05T08:05:00Z").await?;
    let stored = TaskSeries::get_by_id(&mut db, series.id).await?;
    let next = Task::get_by_id(&mut db, stored.latest_task_id.ok_or("missing successor")?).await?;
    assert_eq!(next.due_at, Some("2028-03-12T08:00:00Z".parse()?));
    Ok(())
}

#[tokio::test]
async fn noop_configuration_does_not_reanchor_a_rescheduled_occurrence() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (_, task, series) = successor(&state, user.id).await?;
    let mut changed = edit(&task, RecurrenceChange::Set(config(SeriesEnd::Never)));
    changed.due_at = Some("2028-03-05T08:00:00Z".parse()?);
    save(&state, &task, changed).await?;
    assert_eq!(
        Task::get_by_id(&mut db, task.id).await?.scheduled_at,
        task.scheduled_at
    );
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id).await?.anchor,
        series.anchor
    );
    Ok(())
}

#[tokio::test]
async fn noop_configuration_with_normalized_zone_does_not_block_undo() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (historical, task, _) = successor(&state, user.id).await?;
    let mut rule = config(SeriesEnd::Never);
    rule.time_zone = "europe/warsaw".to_owned();
    save(&state, &task, edit(&task, RecurrenceChange::Set(rule))).await?;
    assert_eq!(
        Task::get_by_id(&mut db, task.id)
            .await?
            .recurrence_modified_at,
        None
    );
    reopen_at(&state, user.id, historical.id, "2028-02-01T10:00:00Z").await?;
    Ok(())
}

#[tokio::test]
async fn changing_limits_preserves_the_existing_count_and_rejects_lower_limits() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (_, task, series) = successor(&state, user.id).await?;
    let error = save(
        &state,
        &task,
        edit(
            &task,
            RecurrenceChange::Set(config(SeriesEnd::After { count: 1 })),
        ),
    )
    .await
    .err()
    .ok_or("expected count rejection")?;
    assert!(matches!(error, TaskError::BadRequest(_)));
    assert_eq!(
        Task::get_by_id(&mut db, task.id)
            .await?
            .recurrence_modified_at,
        None
    );
    save(
        &state,
        &task,
        edit(
            &task,
            RecurrenceChange::Set(config(SeriesEnd::After { count: 2 })),
        ),
    )
    .await?;
    finish_at(&state, user.id, task.id, "2028-02-29T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(series.occurrence_count, 2);
    assert_eq!(series.latest_task_id, Some(task.id));
    Ok(())
}

#[tokio::test]
async fn disabling_and_reenabling_starts_a_new_series_with_a_fresh_limit() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (historical, task, old_series) = successor(&state, user.id).await?;
    let mut disable = edit(&task, RecurrenceChange::Disable);
    disable.due_at = None;
    save(&state, &task, disable).await?;
    let detached = Task::get_by_id(&mut db, task.id).await?;
    assert_eq!(detached.series_id, None);
    assert_eq!(detached.scheduled_at, None);
    assert_eq!(detached.predecessor_task_id, None);
    assert!(
        TaskSeries::get_by_id(&mut db, old_series.id)
            .await?
            .stopped_at
            .is_some()
    );
    assert_eq!(
        Task::get_by_id(&mut db, historical.id).await?.series_id,
        Some(old_series.id)
    );
    let mut enable = edit(
        &detached,
        RecurrenceChange::Set(config(SeriesEnd::After { count: 2 })),
    );
    enable.due_at = Some("2028-06-01T07:00:00Z".parse()?);
    save(&state, &detached, enable).await?;
    let task = Task::get_by_id(&mut db, task.id).await?;
    let new_id = task.series_id.ok_or("missing new series")?;
    assert_ne!(new_id, old_series.id);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, new_id)
            .await?
            .occurrence_count,
        1
    );
    assert_eq!(task.recurrence_modified_at, None);
    assert!(
        reopen_at(&state, user.id, historical.id, "2028-06-01T10:00:00Z")
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn historical_configuration_cannot_be_changed_and_is_not_replaced_in_responses() -> TestResult
{
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (historical, task, _) = successor(&state, user.id).await?;
    assert!(
        save(
            &state,
            &historical,
            edit(&historical, RecurrenceChange::Disable)
        )
        .await
        .is_err()
    );
    assert!(
        save(
            &state,
            &historical,
            edit(&historical, RecurrenceChange::Set(config(SeriesEnd::Never)))
        )
        .await
        .is_err()
    );
    let Json(response) =
        get_task(State(state.clone()), CurrentUser(user), Path(historical.id)).await?;
    assert!(response.recurrence.is_none());
    assert_eq!(response.scheduled_at, historical.scheduled_at);
    let user = User::get_by_id(&mut db, task.owner_id).await?;
    let Json(response) = get_task(State(state.clone()), CurrentUser(user), Path(task.id)).await?;
    assert!(response.recurrence.is_some());
    let user = User::get_by_id(&mut db, task.owner_id).await?;
    let Json(page) = list(
        State(state.clone()),
        CurrentUser(user),
        Query(ListTasksQuery::default()),
    )
    .await?;
    assert!(page.items.iter().any(|task| task.recurrence.is_some()));
    Ok(())
}

#[tokio::test]
async fn http_contract_distinguishes_omitted_null_and_object_configuration() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let token = create_session(&mut db, user.id, Timestamp::now()).await?;
    let cookie = format!("rincuma_session={}", token.expose());
    let app = get_app_router(state.clone());
    let body = r#"{"name":"Wire task","due_at":"2028-01-31T08:00:00Z","recurrence":{"frequency":"monthly","interval":1,"time_zone":"Europe/Warsaw","end":{"mode":"never"}}}"#;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("Cookie", &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(body))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    let task = Task::filter(Task::fields().owner_id().eq(user.id))
        .first()
        .exec(&mut db)
        .await?
        .ok_or("missing wire task")?;
    let series_id = task.series_id;
    for (payload, expected_series) in [
        (
            r#"{"name":"Wire task","due_at":"2028-01-31T08:00:00Z"}"#,
            series_id,
        ),
        (
            r#"{"name":"Wire task","due_at":null,"recurrence":null}"#,
            None,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/tasks/{}", task.id))
                    .header("Cookie", &cookie)
                    .header("Content-Type", "application/json")
                    .body(Body::from(payload))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            Task::get_by_id(&mut db, task.id).await?.series_id,
            expected_series
        );
    }
    let invalid = r#"{"name":"Wire task","due_at":"2028-01-31T08:00:00Z","recurrence":{"frequency":"monthly","interval":1,"time_zone":"Europe/Warsaw","end":{"mode":"after","count":2,"date":"2028-12-31"}}}"#;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("Cookie", &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(invalid))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    Ok(())
}

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in RINCUMA_TEST_POSTGRES_URL"]
async fn postgres_configuration_changes_serialize_with_undo() -> TestResult {
    use crate::{
        config::AppConfig,
        web::task::{handlers::reopen, locking::lock_owned_task, recurrence::apply_recurrence},
    };
    use std::{path::PathBuf, time::Duration};

    let url = std::env::var("RINCUMA_TEST_POSTGRES_URL")?;
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url)
        .await?;
    db.push_schema().await?;
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let user = toasty::create!(User {
        email: "configuration@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let owner_id = user.id;
    let state = AppState::new(AppConfig::for_test(PathBuf::new()), db.clone());
    let (_, Json(created)) =
        create(State(state.clone()), CurrentUser(user), Json(request()?)).await?;
    finish_at(&state, owner_id, created.id, "2028-01-31T08:05:00Z").await?;
    let series_id = created.series_id.ok_or("missing series")?;
    let series = TaskSeries::get_by_id(&mut db, series_id).await?;
    let successor_id = series.latest_task_id.ok_or("missing successor")?;
    let user = User::get_by_id(&mut db, owner_id).await?;

    let mut transaction = db.transaction().await?;
    let mut locked = lock_owned_task(&mut transaction, successor_id, owner_id).await?;
    let mut changed = config(SeriesEnd::After { count: 2 });
    changed.frequency = SeriesFrequency::Daily;
    apply_recurrence(
        &mut transaction,
        &mut locked,
        RecurrenceChange::Set(changed),
        now,
    )
    .await?;
    let competing_state = state.clone();
    let mut undo = tokio::spawn(async move {
        reopen(State(competing_state), CurrentUser(user), Path(created.id)).await
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut undo)
            .await
            .is_err()
    );
    transaction.commit().await?;
    let error = undo.await?.err().ok_or("expected changed rule rejection")?;
    assert!(matches!(error, TaskError::Conflict(..)));
    let stored = TaskSeries::get_by_id(&mut db, series_id).await?;
    assert_eq!(stored.frequency, SeriesFrequency::Daily);
    assert_eq!(stored.end, SeriesEnd::After { count: 2 });
    assert_eq!(stored.occurrence_count, 2);

    let current = Task::get_by_id(&mut db, successor_id).await?;
    save(&state, &current, edit(&current, RecurrenceChange::Disable)).await?;
    let current = Task::get_by_id(&mut db, successor_id).await?;
    save(
        &state,
        &current,
        edit(&current, RecurrenceChange::Set(config(SeriesEnd::Never))),
    )
    .await?;
    let new_series_id = Task::get_by_id(&mut db, successor_id)
        .await?
        .series_id
        .ok_or("missing new series")?;
    assert_ne!(new_series_id, series_id);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, new_series_id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}
