#![allow(clippy::panic_in_result_fn)]

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use jiff::{Timestamp, civil::date};

use super::complete_locked;
use crate::{
    app_state::AppState,
    db::{Project, SeriesEnd, SeriesFrequency, Tag, Task, TaskPriority, TaskSeries, TaskTag, User},
    web::{
        auth::CurrentUser,
        task::{
            TaskError,
            handlers::{complete, create, reopen, update},
            request::SaveTaskRequest,
        },
        test_support::state_with_user,
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn recurring(
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

async fn finish_at(state: &AppState, owner_id: i64, id: i64, now: &str) -> Result<Task, TaskError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let mut task = Task::lock_owned(&mut transaction, id, owner_id)
        .await?
        .ok_or_else(TaskError::related_resource_not_found)?;
    let tags = TaskTag::filter_by_task_id(id)
        .exec(&mut transaction)
        .await?;
    let tag_ids: Vec<i64> = tags.into_iter().map(|tag| tag.tag_id).collect();
    let now = now.parse().map_err(|_| TaskError::bad_request())?;
    complete_locked(&mut transaction, &mut task, &tag_ids, now).await?;
    transaction.commit().await?;
    Ok(task)
}

async fn occurrences(executor: &mut dyn toasty::Executor, id: i64) -> toasty::Result<Vec<Task>> {
    Task::filter(Task::fields().series_id().eq(Some(id)))
        .order_by(Task::fields().id().asc())
        .exec(executor)
        .await
}

#[tokio::test]
async fn copies_current_fields_relations_and_tags_into_a_fresh_occurrence() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (mut original, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let project = toasty::create!(Project {
        owner_id: user.id,
        name: "Archived project".to_owned(),
        archived_at: Some(now),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let parent = toasty::create!(Task {
        owner_id: user.id,
        name: "Parent".to_owned(),
        project_id: Some(project.id),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let tag = toasty::create!(Tag {
        owner_id: user.id,
        name: "Tag".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    toasty::create!(TaskTag {
        task_id: original.id,
        tag_id: tag.id
    })
    .exec(&mut db)
    .await?;
    toasty::update!(original {
        name: "Updated name".to_owned(),
        description: Some("Updated description".to_owned()),
        priority: TaskPriority::Urgent,
        project_id: Some(project.id),
        parent_id: Some(parent.id),
        due_at: Some("2030-01-01T00:00:00Z".parse::<Timestamp>()?),
        recurrence_modified_at: Some(now),
    })
    .exec(&mut db)
    .await?;

    finish_at(&state, user.id, original.id, "2028-01-31T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let successor =
        Task::get_by_id(&mut db, series.latest_task_id.ok_or("missing successor")?).await?;
    assert_ne!(successor.id, original.id);
    assert_eq!(successor.name, original.name);
    assert_eq!(successor.description, original.description);
    assert_eq!(successor.priority, original.priority);
    assert_eq!(successor.project_id, original.project_id);
    assert_eq!(successor.parent_id, original.parent_id);
    assert_eq!(successor.owner_id, original.owner_id);
    assert_eq!(successor.series_id, Some(series.id));
    assert_eq!(successor.due_at, Some("2028-02-29T08:00:00Z".parse()?));
    assert_eq!(successor.scheduled_at, successor.due_at);
    assert_eq!(successor.completed_at, None);
    assert_eq!(successor.recurrence_modified_at, None);
    assert_eq!(successor.created_at, "2028-01-31T08:05:00Z".parse()?);
    assert_eq!(successor.updated_at, successor.created_at);
    assert_eq!(series.occurrence_count, 2);
    let tags = TaskTag::filter_by_task_id(successor.id)
        .exec(&mut db)
        .await?;
    assert_eq!(
        tags.iter().map(|tag| tag.tag_id).collect::<Vec<_>>(),
        vec![tag.id]
    );
    let children = Task::filter(Task::fields().parent_id().eq(Some(successor.id)))
        .exec(&mut db)
        .await?;
    assert!(children.is_empty());
    Ok(())
}

#[tokio::test]
async fn repeated_completion_does_not_create_or_count_another_successor() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let first = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z").await?;
    let repeated = finish_at(&state, user.id, task.id, "2028-06-01T10:00:00Z").await?;
    assert_eq!(repeated.completed_at, first.completed_at);
    assert_eq!(repeated.updated_at, first.updated_at);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 2);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        2
    );
    Ok(())
}

#[tokio::test]
async fn late_completion_skips_elapsed_slots_without_consuming_the_limit() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, user.id, SeriesEnd::After { count: 2 }).await?;
    finish_at(&state, user.id, task.id, "2028-05-15T10:00:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let successor =
        Task::get_by_id(&mut db, series.latest_task_id.ok_or("missing successor")?).await?;
    assert_eq!(successor.due_at, Some("2028-05-31T07:00:00Z".parse()?));
    assert_eq!(series.occurrence_count, 2);
    finish_at(&state, user.id, successor.id, "2028-05-31T07:05:00Z").await?;
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 2);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .latest_task_id,
        Some(successor.id)
    );
    Ok(())
}

#[tokio::test]
async fn date_and_count_end_modes_complete_the_final_task_without_a_successor() -> TestResult {
    for end in [
        SeriesEnd::After { count: 1 },
        SeriesEnd::Through {
            date: date(2028, 1, 31),
        },
    ] {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let (task, series) = recurring(&mut db, user.id, end).await?;
        let completed = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z").await?;
        assert!(completed.completed_at.is_some());
        assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
        let series = TaskSeries::get_by_id(&mut db, series.id).await?;
        assert_eq!(series.occurrence_count, 1);
        assert_eq!(series.latest_task_id, Some(task.id));
        assert_eq!(series.stopped_at, None);
    }
    Ok(())
}

#[tokio::test]
async fn stopped_series_does_not_advance() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, mut series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    toasty::update!(series {
        stopped_at: Some("2028-01-31T08:00:00Z".parse::<Timestamp>()?)
    })
    .exec(&mut db)
    .await?;
    let completed = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z").await?;
    assert!(completed.completed_at.is_some());
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}

#[tokio::test]
async fn invalid_schedule_rolls_back_completion_and_series_changes() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, mut series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    toasty::update!(series {
        time_zone: "Not/A_Zone".to_owned()
    })
    .exec(&mut db)
    .await?;
    let error = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z")
        .await
        .err()
        .ok_or("expected invalid schedule")?;
    assert!(matches!(error, TaskError::BadRequest(_)));
    assert_eq!(Task::get_by_id(&mut db, task.id).await?.completed_at, None);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}

#[tokio::test]
async fn tasks_with_children_are_rejected_without_copying_or_completing_them() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    toasty::create!(Task {
        owner_id: user.id,
        name: "Child".to_owned(),
        parent_id: Some(task.id),
        created_at: task.created_at,
        updated_at: task.created_at,
    })
    .exec(&mut db)
    .await?;
    let error = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z")
        .await
        .err()
        .ok_or("expected hierarchy rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    assert_eq!(Task::get_by_id(&mut db, task.id).await?.completed_at, None);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn another_owners_series_cannot_be_advanced() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, mut series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: task.created_at,
        updated_at: task.created_at,
    })
    .exec(&mut db)
    .await?;
    toasty::update!(series { owner_id: other.id })
        .exec(&mut db)
        .await?;
    let error = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z")
        .await
        .err()
        .ok_or("expected ownership rejection")?;
    assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
    assert_eq!(Task::get_by_id(&mut db, task.id).await?.completed_at, None);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}

#[tokio::test]
async fn a_tag_copy_failure_rolls_back_the_inserted_successor() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let tag = toasty::create!(Tag {
        owner_id: user.id,
        name: "Tag".to_owned(),
        created_at: task.created_at,
        updated_at: task.created_at,
    })
    .exec(&mut db)
    .await?;
    toasty::create!(TaskTag {
        task_id: task.id,
        tag_id: tag.id
    })
    .exec(&mut db)
    .await?;
    toasty::sql::statement(format!(
        "CREATE TRIGGER fail_successor_tags BEFORE INSERT ON task_tags WHEN NEW.task_id != {} BEGIN SELECT RAISE(ABORT, 'forced tag copy failure'); END",
        task.id,
    )).exec(&mut db).await?;
    let error = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z")
        .await
        .err()
        .ok_or("expected copy failure")?;
    assert!(matches!(error, TaskError::Database(..)));
    assert_eq!(Task::get_by_id(&mut db, task.id).await?.completed_at, None);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(series.occurrence_count, 1);
    assert_eq!(series.latest_task_id, Some(task.id));
    Ok(())
}

#[tokio::test]
async fn create_and_update_cannot_attach_children_to_a_recurring_parent() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (parent, _) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let child = toasty::create!(Task {
        owner_id: user.id,
        name: "Child".to_owned(),
        created_at: parent.created_at,
        updated_at: parent.created_at,
    })
    .exec(&mut db)
    .await?;
    let request = || SaveTaskRequest {
        name: "Child".to_owned(),
        description: None,
        due_at: None,
        priority: None,
        project_id: None,
        parent_id: Some(parent.id),
        tag_ids: Vec::new(),
    };
    let error = create(State(state.clone()), CurrentUser(user), Json(request()))
        .await
        .err()
        .ok_or("expected recurring parent rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    let user = User::get_by_id(&mut db, parent.owner_id).await?;
    let error = update(
        State(state.clone()),
        CurrentUser(user),
        Path(child.id),
        Json(request()),
    )
    .await
    .err()
    .ok_or("expected recurring parent rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    assert_eq!(Task::get_by_id(&mut db, child.id).await?.parent_id, None);
    Ok(())
}

#[tokio::test]
async fn unowned_projects_and_tags_are_never_copied_to_a_successor() -> TestResult {
    for foreign_project in [false, true] {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let (mut task, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
        let other = toasty::create!(User {
            email: "other@example.com".to_owned(),
            password_hash: "unused".to_owned(),
            created_at: task.created_at,
            updated_at: task.created_at,
        })
        .exec(&mut db)
        .await?;
        if foreign_project {
            let project = toasty::create!(Project {
                owner_id: other.id,
                name: "Foreign project".to_owned(),
                created_at: task.created_at,
                updated_at: task.created_at,
            })
            .exec(&mut db)
            .await?;
            toasty::update!(task {
                project_id: Some(project.id)
            })
            .exec(&mut db)
            .await?;
        } else {
            let tag = toasty::create!(Tag {
                owner_id: other.id,
                name: "Foreign tag".to_owned(),
                created_at: task.created_at,
                updated_at: task.created_at,
            })
            .exec(&mut db)
            .await?;
            toasty::create!(TaskTag {
                task_id: task.id,
                tag_id: tag.id
            })
            .exec(&mut db)
            .await?;
        }
        let error = finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z")
            .await
            .err()
            .ok_or("expected relation ownership rejection")?;
        assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
        assert_eq!(Task::get_by_id(&mut db, task.id).await?.completed_at, None);
        assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    }
    Ok(())
}

#[tokio::test]
async fn completion_route_is_idempotent_and_reopening_cannot_leave_a_successor_behind() -> TestResult
{
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let Json(first) = complete(State(state.clone()), CurrentUser(user), Path(task.id)).await?;
    let user = User::get_by_id(&mut db, task.owner_id).await?;
    let Json(repeated) = complete(State(state.clone()), CurrentUser(user), Path(task.id)).await?;
    assert_eq!(first.completed_at, repeated.completed_at);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 2);
    let user = User::get_by_id(&mut db, task.owner_id).await?;
    let error = reopen(State(state.clone()), CurrentUser(user), Path(task.id))
        .await
        .err()
        .ok_or("expected reopening rejection")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    assert_eq!(
        Task::get_by_id(&mut db, task.id).await?.completed_at,
        first.completed_at
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in RINCUMA_TEST_POSTGRES_URL"]
async fn postgres_competing_completions_create_exactly_one_successor() -> TestResult {
    use crate::config::AppConfig;
    use std::{path::PathBuf, time::Duration};

    let url = std::env::var("RINCUMA_TEST_POSTGRES_URL")?;
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url)
        .await?;
    db.push_schema().await?;
    let now: Timestamp = "2028-01-31T08:00:00Z".parse()?;
    let user = toasty::create!(User {
        email: "concurrency@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let state = AppState::new(AppConfig::for_test(PathBuf::new()), db.clone());
    let (task, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;

    let mut first_db = db.clone();
    let mut transaction = first_db.transaction().await?;
    let mut locked = Task::lock_owned(&mut transaction, task.id, user.id)
        .await?
        .ok_or("missing task")?;
    let second_state = state.clone();
    let task_id = task.id;
    let (started, waiting) = tokio::sync::oneshot::channel();
    let mut competing = tokio::spawn(async move {
        let _ = started.send(());
        complete(State(second_state), CurrentUser(user), Path(task_id)).await
    });
    waiting.await?;
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut competing)
            .await
            .is_err()
    );

    complete_locked(&mut transaction, &mut locked, &[], now).await?;
    transaction.commit().await?;
    let Json(response) = competing.await??;
    assert_eq!(response.completed_at, Some(now));
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 2);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        2
    );
    Ok(())
}
