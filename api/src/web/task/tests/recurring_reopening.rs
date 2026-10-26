#![allow(clippy::panic_in_result_fn)]

use crate::web::task::recurrence::RecurrenceChange;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use jiff::civil::date;

use crate::{
    app_state::AppState,
    db::{Project, SeriesEnd, Tag, Task, TaskPriority, TaskSeries, TaskTag, User},
    web::{
        auth::CurrentUser,
        task::{
            TaskError,
            handlers::{delete, update},
            recurring_support::{finish_at, occurrences, recurring, reopen_at},
            request::SaveTaskRequest,
        },
        test_support::state_with_user,
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn pair(
    state: &AppState,
    owner_id: i64,
) -> Result<(Task, Task, TaskSeries), Box<dyn std::error::Error>> {
    let mut db = state.db().clone();
    let (task, series) = recurring(&mut db, owner_id, SeriesEnd::After { count: 3 }).await?;
    let task = finish_at(state, owner_id, task.id, "2028-01-31T08:05:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let successor =
        Task::get_by_id(&mut db, series.latest_task_id.ok_or("missing successor")?).await?;
    Ok((task, successor, series))
}

fn request(task: &Task, tag_ids: Vec<i64>) -> SaveTaskRequest {
    SaveTaskRequest {
        recurrence: RecurrenceChange::Keep,
        name: task.name.clone(),
        description: task.description.clone(),
        due_at: task.due_at,
        priority: Some(task.priority),
        project_id: task.project_id,
        parent_id: task.parent_id,
        tag_ids,
    }
}

async fn assert_blocked(state: &AppState, owner_id: i64, id: i64) -> TestResult {
    let error = reopen_at(state, owner_id, id, "2028-02-01T10:00:00Z")
        .await
        .err()
        .ok_or("expected reopening to be blocked")?;
    assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    let mut db = state.db().clone();
    assert!(Task::get_by_id(&mut db, id).await?.completed_at.is_some());
    Ok(())
}

#[tokio::test]
async fn undo_deletes_only_the_untouched_successor_and_restores_its_count() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (original, successor, series) = pair(&state, user.id).await?;
    assert_eq!(successor.predecessor_task_id, Some(original.id));
    let reopened = reopen_at(&state, user.id, original.id, "2028-02-01T10:00:00Z").await?;
    assert_eq!(reopened.completed_at, None);
    assert_eq!(reopened.name, original.name);
    assert_eq!(reopened.description, original.description);
    assert_eq!(reopened.due_at, original.due_at);
    assert_eq!(reopened.scheduled_at, original.scheduled_at);
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    assert!(
        Task::filter_by_id(successor.id)
            .first()
            .exec(&mut db)
            .await?
            .is_none()
    );
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(series.latest_task_id, Some(original.id));
    assert_eq!(series.occurrence_count, 1);

    finish_at(&state, user.id, original.id, "2028-03-15T10:00:00Z").await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    let next =
        Task::get_by_id(&mut db, series.latest_task_id.ok_or("missing replacement")?).await?;
    assert_ne!(next.id, successor.id);
    assert_eq!(next.due_at, Some("2028-03-31T07:00:00Z".parse()?));
    assert_eq!(series.occurrence_count, 2);
    Ok(())
}

#[tokio::test]
async fn repeating_undo_does_not_release_the_limit_twice() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, _, series) = pair(&state, user.id).await?;
    let first = reopen_at(&state, user.id, task.id, "2028-02-01T10:00:00Z").await?;
    let again = reopen_at(&state, user.id, task.id, "2028-02-02T10:00:00Z").await?;
    assert_eq!(again.updated_at, first.updated_at);
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .occurrence_count,
        1
    );
    Ok(())
}

#[tokio::test]
async fn only_the_latest_completed_occurrence_can_be_undone() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (first, second, series) = pair(&state, user.id).await?;
    finish_at(&state, user.id, second.id, "2028-02-29T08:05:00Z").await?;
    assert_blocked(&state, user.id, first.id).await?;
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 3);
    reopen_at(&state, user.id, second.id, "2028-03-01T10:00:00Z").await?;
    reopen_at(&state, user.id, first.id, "2028-03-01T10:05:00Z").await?;
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn a_naturally_ended_series_can_reopen_its_last_occurrence() -> TestResult {
    for end in [
        SeriesEnd::After { count: 1 },
        SeriesEnd::Through {
            date: date(2028, 1, 31),
        },
    ] {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let (task, series) = recurring(&mut db, user.id, end).await?;
        finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z").await?;
        let task = reopen_at(&state, user.id, task.id, "2028-02-01T10:00:00Z").await?;
        assert_eq!(task.completed_at, None);
        assert_eq!(
            TaskSeries::get_by_id(&mut db, series.id)
                .await?
                .occurrence_count,
            1
        );
        finish_at(&state, user.id, task.id, "2028-02-01T10:05:00Z").await?;
        assert_eq!(occurrences(&mut db, series.id).await?.len(), 1);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Edit {
    Name,
    Description,
    Due,
    Priority,
    Project,
    Parent,
    Tags,
}

async fn edit_request(
    executor: &mut dyn toasty::Executor,
    task: &Task,
    edit: Edit,
) -> Result<SaveTaskRequest, Box<dyn std::error::Error>> {
    let mut request = request(task, Vec::new());
    match edit {
        Edit::Name => request.name = "Changed name".to_owned(),
        Edit::Description => request.description = Some("Changed description".to_owned()),
        Edit::Due => request.due_at = Some("2028-03-05T10:00:00Z".parse()?),
        Edit::Priority => request.priority = Some(TaskPriority::Urgent),
        Edit::Project => {
            let project = toasty::create!(Project {
                owner_id: task.owner_id,
                name: "Project".to_owned(),
                created_at: task.created_at,
                updated_at: task.created_at
            })
            .exec(executor)
            .await?;
            request.project_id = Some(project.id);
        }
        Edit::Parent => {
            let parent = toasty::create!(Task {
                owner_id: task.owner_id,
                name: "Parent".to_owned(),
                created_at: task.created_at,
                updated_at: task.created_at
            })
            .exec(executor)
            .await?;
            request.parent_id = Some(parent.id);
        }
        Edit::Tags => {
            let tag = toasty::create!(Tag {
                owner_id: task.owner_id,
                name: "Tag".to_owned(),
                created_at: task.created_at,
                updated_at: task.created_at
            })
            .exec(executor)
            .await?;
            request.tag_ids = vec![tag.id];
        }
    }
    Ok(request)
}

#[tokio::test]
async fn each_meaningful_successor_edit_blocks_undo() -> TestResult {
    for edit in [
        Edit::Name,
        Edit::Description,
        Edit::Due,
        Edit::Priority,
        Edit::Project,
        Edit::Parent,
        Edit::Tags,
    ] {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let (original, successor, series) = pair(&state, user.id).await?;
        let request = edit_request(&mut db, &successor, edit).await?;
        let _ = update(
            State(state.clone()),
            CurrentUser(user),
            Path(successor.id),
            Json(request),
        )
        .await?;
        let successor = Task::get_by_id(&mut db, successor.id).await?;
        assert!(successor.recurrence_modified_at.is_some());
        assert_blocked(&state, original.owner_id, original.id).await?;
        assert_eq!(occurrences(&mut db, series.id).await?.len(), 2);
        assert_eq!(
            TaskSeries::get_by_id(&mut db, series.id)
                .await?
                .occurrence_count,
            2
        );
    }
    Ok(())
}

#[tokio::test]
async fn noop_save_and_tag_reordering_do_not_block_undo() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (original, successor, _) = pair(&state, user.id).await?;
    let mut tags = Vec::new();
    for name in ["First", "Second"] {
        let tag = toasty::create!(Tag {
            owner_id: user.id,
            name: name.to_owned(),
            created_at: successor.created_at,
            updated_at: successor.created_at
        })
        .exec(&mut db)
        .await?;
        toasty::create!(TaskTag {
            task_id: successor.id,
            tag_id: tag.id
        })
        .exec(&mut db)
        .await?;
        tags.push(tag.id);
    }
    tags.reverse();
    let _ = update(
        State(state.clone()),
        CurrentUser(user),
        Path(successor.id),
        Json(request(&successor, tags.clone())),
    )
    .await?;
    assert_eq!(
        Task::get_by_id(&mut db, successor.id)
            .await?
            .recurrence_modified_at,
        None
    );
    reopen_at(
        &state,
        original.owner_id,
        original.id,
        "2028-02-01T10:00:00Z",
    )
    .await?;
    assert!(
        TaskTag::filter_by_task_id(successor.id)
            .exec(&mut db)
            .await?
            .is_empty()
    );
    for tag_id in tags {
        assert_eq!(Tag::get_by_id(&mut db, tag_id).await?.id, tag_id);
    }
    Ok(())
}

#[tokio::test]
async fn reverting_an_edit_does_not_clear_the_durable_change_marker() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (original, successor, _) = pair(&state, user.id).await?;
    let changed = edit_request(&mut db, &successor, Edit::Name).await?;
    let _ = update(
        State(state.clone()),
        CurrentUser(user),
        Path(successor.id),
        Json(changed),
    )
    .await?;
    let user = User::get_by_id(&mut db, original.owner_id).await?;
    let _ = update(
        State(state.clone()),
        CurrentUser(user),
        Path(successor.id),
        Json(request(&successor, Vec::new())),
    )
    .await?;
    assert_blocked(&state, original.owner_id, original.id).await?;
    Ok(())
}

#[tokio::test]
async fn an_edit_blocks_undo_even_with_equal_audit_timestamps() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (original, mut successor, _) = pair(&state, user.id).await?;
    toasty::update!(successor {
        recurrence_modified_at: Some(successor.created_at)
    })
    .exec(&mut db)
    .await?;
    assert_eq!(successor.created_at, successor.updated_at);
    assert_blocked(&state, user.id, original.id).await
}

#[tokio::test]
async fn detached_or_reassigned_successors_cannot_be_deleted_by_undo() -> TestResult {
    for reassign in [false, true] {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let (original, mut successor, _) = pair(&state, user.id).await?;
        let new_series_id = if reassign {
            let (_, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
            Some(series.id)
        } else {
            None
        };
        toasty::update!(successor {
            series_id: new_series_id
        })
        .exec(&mut db)
        .await?;
        assert_blocked(&state, user.id, original.id).await?;
        assert_eq!(
            Task::get_by_id(&mut db, successor.id).await?.series_id,
            new_series_id
        );
    }
    Ok(())
}

#[tokio::test]
async fn deleting_the_active_occurrence_stops_the_series_and_preserves_history() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (original, successor, series) = pair(&state, user.id).await?;
    let _ = delete(State(state.clone()), CurrentUser(user), Path(successor.id)).await?;
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert!(series.stopped_at.is_some());
    assert!(
        Task::get_by_id(&mut db, original.id)
            .await?
            .completed_at
            .is_some()
    );
    assert_blocked(&state, original.owner_id, original.id).await
}

#[tokio::test]
async fn deleted_completed_history_cannot_make_an_unrelated_successor_undoable() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (first, second, series) = pair(&state, user.id).await?;
    finish_at(&state, user.id, second.id, "2028-02-29T08:05:00Z").await?;
    let _ = delete(State(state.clone()), CurrentUser(user), Path(second.id)).await?;
    assert_blocked(&state, first.owner_id, first.id).await?;
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn undo_rollback_restores_the_deleted_successor_and_count() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (original, successor, series) = pair(&state, user.id).await?;
    toasty::sql::statement("CREATE TRIGGER fail_undo_count BEFORE UPDATE ON task_series WHEN NEW.occurrence_count < OLD.occurrence_count BEGIN SELECT RAISE(ABORT, 'forced undo count failure'); END").exec(&mut db).await?;
    let error = reopen_at(&state, user.id, original.id, "2028-02-01T10:00:00Z")
        .await
        .err()
        .ok_or("expected rollback")?;
    assert!(matches!(error, TaskError::Database(..)));
    assert!(
        Task::get_by_id(&mut db, original.id)
            .await?
            .completed_at
            .is_some()
    );
    assert_eq!(
        Task::get_by_id(&mut db, successor.id).await?.id,
        successor.id
    );
    let series = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(series.occurrence_count, 2);
    assert_eq!(series.latest_task_id, Some(successor.id));
    Ok(())
}

#[tokio::test]
async fn undo_of_a_final_occurrence_does_not_revert_series_configuration() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (task, mut series) = recurring(&mut db, user.id, SeriesEnd::After { count: 1 }).await?;
    finish_at(&state, user.id, task.id, "2028-01-31T08:05:00Z").await?;
    toasty::update!(series {
        end: SeriesEnd::Never,
        interval: 2
    })
    .exec(&mut db)
    .await?;
    reopen_at(&state, user.id, task.id, "2028-02-01T10:00:00Z").await?;
    let stored = TaskSeries::get_by_id(&mut db, series.id).await?;
    assert_eq!(stored.end, SeriesEnd::Never);
    assert_eq!(stored.interval, 2);
    Ok(())
}
