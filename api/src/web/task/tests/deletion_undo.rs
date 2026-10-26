use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use jiff::{SignedDuration, Timestamp};
use tower::ServiceExt;

use crate::{
    app_state::AppState,
    auth::session::create_session,
    db::{Project, SeriesEnd, Tag, Task, TaskPriority, TaskSeries, TaskTag, User},
    web::{
        router::get_app_router,
        task::{
            TaskError,
            deletion::DeletionReceipt,
            recurring_support::{finish_at, occurrences, recurring},
            restoration::{delete_with_undo, restore_deletion},
        },
        test_support::state_with_user,
    },
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn task(db: &mut toasty::Db, owner_id: i64, parent_id: Option<i64>) -> toasty::Result<Task> {
    let now = Timestamp::now();
    toasty::create!(Task {
        owner_id,
        parent_id,
        name: "Saved task".to_owned(),
        description: Some("Saved description".to_owned()),
        priority: TaskPriority::High,
        due_at: Some(now),
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

async fn remove(
    state: &AppState,
    owner_id: i64,
    id: i64,
    now: Timestamp,
) -> Result<DeletionReceipt, TaskError> {
    let mut db = state.db().clone();
    let mut tx = db.transaction().await?;
    let deletion = delete_with_undo(&mut tx, id, owner_id, now).await?;
    tx.commit().await?;
    Ok(deletion)
}

async fn restore(
    state: &AppState,
    owner_id: i64,
    deletion: &DeletionReceipt,
    now: Timestamp,
) -> Result<(), TaskError> {
    let mut db = state.db().clone();
    let mut tx = db.transaction().await?;
    restore_deletion(&mut tx, deletion.id, owner_id, deletion.deleted_at, now).await?;
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn restores_original_subtree_ids_fields_and_tags_once() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let parent = task(&mut db, user.id, None).await?;
    let root = task(&mut db, user.id, Some(parent.id)).await?;
    let child = task(&mut db, user.id, Some(root.id)).await?;
    let mut grandchild = task(&mut db, user.id, Some(child.id)).await?;
    let now = Timestamp::now();
    toasty::update!(grandchild {
        completed_at: Some(now)
    })
    .exec(&mut db)
    .await?;
    let tag = toasty::create!(Tag {
        name: "Retained tag".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    for id in [root.id, child.id] {
        toasty::create!(TaskTag {
            task_id: id,
            tag_id: tag.id
        })
        .exec(&mut db)
        .await?;
    }
    let deletion = remove(&state, user.id, root.id, now).await?;
    assert!(Task::get_owned(&mut db, root.id, user.id).await?.is_none());
    restore(&state, user.id, &deletion, now).await?;
    for original in [&root, &child, &grandchild] {
        let restored = Task::get_by_id(&mut db, &original.id).await?;
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.description, original.description);
        assert_eq!(restored.priority, original.priority);
        assert_eq!(restored.parent_id, original.parent_id);
        assert_eq!(restored.due_at, original.due_at);
        assert_eq!(restored.completed_at, original.completed_at);
        assert_eq!(restored.created_at, original.created_at);
        assert_eq!(restored.updated_at, original.updated_at);
    }
    assert_eq!(
        TaskTag::filter_by_task_id(child.id)
            .exec(&mut db)
            .await?
            .first()
            .map(|link| link.tag_id),
        Some(tag.id)
    );
    assert!(
        Task::get_by_id(&mut db, deletion.id)
            .await?
            .deletion_root_id
            .is_none()
    );
    assert!(matches!(
        restore(&state, user.id, &deletion, now).await,
        Err(TaskError::RelatedResourceNotFound(_))
    ));
    Ok(())
}

#[tokio::test]
async fn restores_active_recurring_subtask_and_resumes_its_series() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let root = task(&mut db, user.id, None).await?;
    let (first, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    finish_at(&state, user.id, first.id, "2028-01-31T08:00:00Z").await?;
    let mut active = occurrences(&mut db, series.id)
        .await?
        .pop()
        .ok_or("missing successor")?;
    toasty::update!(active {
        parent_id: Some(root.id)
    })
    .exec(&mut db)
    .await?;
    let before = TaskSeries::get_by_id(&mut db, &series.id).await?;
    let now: Timestamp = "2028-02-01T08:00:00Z".parse()?;
    let deletion = remove(&state, user.id, root.id, now).await?;
    assert!(
        TaskSeries::get_by_id(&mut db, &series.id)
            .await?
            .stopped_at
            .is_some()
    );
    assert!(Task::get_owned(&mut db, first.id, user.id).await?.is_some());
    restore(&state, user.id, &deletion, now).await?;
    let resumed = TaskSeries::get_by_id(&mut db, &series.id).await?;
    assert_eq!(resumed.stopped_at, before.stopped_at);
    assert_eq!(resumed.updated_at, now);
    assert_eq!(resumed.occurrence_count, before.occurrence_count);
    assert_eq!(resumed.latest_task_id, Some(active.id));
    let restored = Task::get_by_id(&mut db, &active.id).await?;
    assert_eq!(restored.scheduled_at, active.scheduled_at);
    assert_eq!(restored.predecessor_task_id, Some(first.id));
    finish_at(&state, user.id, active.id, "2028-02-29T08:00:00Z").await?;
    assert_eq!(occurrences(&mut db, series.id).await?.len(), 3);
    Ok(())
}

#[tokio::test]
async fn expired_undo_keeps_rows_deleted_while_unexpired_undo_works() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let first = task(&mut db, user.id, None).await?;
    let second = task(&mut db, user.id, None).await?;
    let now = Timestamp::now();
    let older = remove(&state, user.id, first.id, now).await?;
    let later = now.checked_add(SignedDuration::from_secs(60))?;
    let newer = remove(&state, user.id, second.id, later).await?;
    assert!(matches!(
        restore(&state, user.id, &older, older.expires_at).await,
        Err(TaskError::Conflict(..))
    ));
    assert!(
        Task::get_by_id(&mut db, first.id)
            .await?
            .deleted_at
            .is_some()
    );
    assert!(
        Task::get_by_id(&mut db, older.id)
            .await?
            .deleted_at
            .is_some()
    );
    restore(&state, user.id, &newer, older.expires_at).await?;
    Ok(())
}

#[tokio::test]
async fn rejects_changed_series_without_partial_restoration() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let root = task(&mut db, user.id, None).await?;
    let (mut active, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    toasty::update!(active {
        parent_id: Some(root.id)
    })
    .exec(&mut db)
    .await?;
    let now = Timestamp::now();
    let deletion = remove(&state, user.id, root.id, now).await?;
    let changed_at = now.checked_add(SignedDuration::from_secs(1))?;
    toasty::update!(TaskSeries::filter_by_id(series.id) { updated_at: changed_at })
        .exec(&mut db)
        .await?;
    assert!(matches!(
        restore(&state, user.id, &deletion, changed_at).await,
        Err(TaskError::Conflict(..))
    ));
    assert!(Task::get_owned(&mut db, root.id, user.id).await?.is_none());
    assert!(
        Task::get_owned(&mut db, active.id, user.id)
            .await?
            .is_none()
    );
    assert!(
        Task::get_by_id(&mut db, deletion.id)
            .await?
            .deleted_at
            .is_some()
    );
    Ok(())
}

#[tokio::test]
async fn restores_current_relations_but_rejects_incompatible_parent() -> TestResult {
    for changed in ["tag", "project", "parent"] {
        let (state, user) = state_with_user().await?;
        let mut db = state.db().clone();
        let now = Timestamp::now();
        let project = toasty::create!(Project {
            name: "Project".to_owned(),
            owner_id: user.id,
            created_at: now,
            updated_at: now
        })
        .exec(&mut db)
        .await?;
        let mut parent = task(&mut db, user.id, None).await?;
        let child = task(&mut db, user.id, Some(parent.id)).await?;
        for id in [parent.id, child.id] {
            toasty::update!(Task::filter_by_id(id) { project_id: Some(project.id) })
                .exec(&mut db)
                .await?;
        }
        let tag = toasty::create!(Tag {
            name: "Tag".to_owned(),
            owner_id: user.id,
            created_at: now,
            updated_at: now
        })
        .exec(&mut db)
        .await?;
        toasty::create!(TaskTag {
            task_id: child.id,
            tag_id: tag.id
        })
        .exec(&mut db)
        .await?;
        let deletion = remove(&state, user.id, child.id, now).await?;
        match changed {
            "tag" => Tag::filter_by_id(tag.id).delete().exec(&mut db).await?,
            "project" => {
                Project::filter_by_id(project.id)
                    .delete()
                    .exec(&mut db)
                    .await?
            }
            _ => {
                toasty::update!(parent { project_id: None })
                    .exec(&mut db)
                    .await?
            }
        }
        let result = restore(&state, user.id, &deletion, now).await;
        if changed == "parent" {
            assert!(matches!(result, Err(TaskError::Conflict(..))));
            assert!(Task::get_owned(&mut db, child.id, user.id).await?.is_none());
        } else {
            result?;
            let restored = Task::get_owned(&mut db, child.id, user.id)
                .await?
                .ok_or("missing restored task")?;
            if changed == "tag" {
                assert!(
                    TaskTag::filter_by_task_id(child.id)
                        .exec(&mut db)
                        .await?
                        .is_empty()
                );
            } else {
                assert!(restored.project_id.is_none());
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn routes_require_owner_and_restore_after_restarting_app_state() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let selected = task(&mut db, user.id, None).await?;
    let now = Timestamp::now();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let owner_token = create_session(&mut db, user.id, now).await?;
    let other_token = create_session(&mut db, other.id, now).await?;
    let app = get_app_router(state.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/tasks/{}", selected.id))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/tasks/{}", selected.id))
                .header(
                    "Cookie",
                    format!("rincuma_session={}", other_token.expose()),
                )
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/tasks/{}", selected.id))
                .header(
                    "Cookie",
                    format!("rincuma_session={}", owner_token.expose()),
                )
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let value: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 65536).await?)?;
    let root_id = value
        .get("task_id")
        .and_then(serde_json::Value::as_i64)
        .ok_or("missing task ID")?;
    assert!(value.get("expires_at").is_some());
    let deleted_at = value
        .get("deleted_at")
        .ok_or("missing deletion timestamp")?;
    let restore_body = serde_json::json!({"deleted_at": deleted_at}).to_string();
    let fresh_state = AppState::new(
        crate::config::AppConfig::for_test(std::path::PathBuf::new()),
        db.clone(),
    );
    let app = get_app_router(fresh_state);
    for (token, status) in [
        (None, StatusCode::UNAUTHORIZED),
        (Some(other_token.expose()), StatusCode::NOT_FOUND),
        (Some(owner_token.expose()), StatusCode::NO_CONTENT),
        (Some(owner_token.expose()), StatusCode::NOT_FOUND),
    ] {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!("/tasks/{root_id}/restore"));
        if let Some(token) = token {
            request = request.header("Cookie", format!("rincuma_session={token}"));
        }
        assert_eq!(
            app.clone()
                .oneshot(
                    request
                        .header("Content-Type", "application/json")
                        .body(Body::from(restore_body.clone()))?
                )
                .await?
                .status(),
            status
        );
    }
    assert!(
        Task::get_owned(&mut db, selected.id, user.id)
            .await?
            .is_some()
    );
    Ok(())
}

#[tokio::test]
async fn undo_restores_only_its_group_and_requires_a_visible_parent() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let parent = task(&mut db, user.id, None).await?;
    let child = task(&mut db, user.id, Some(parent.id)).await?;
    let now = Timestamp::now();
    let child_deletion = remove(&state, user.id, child.id, now).await?;
    let parent_deletion = remove(&state, user.id, parent.id, now).await?;
    assert!(matches!(
        restore(&state, user.id, &child_deletion, now).await,
        Err(TaskError::Conflict(..))
    ));
    restore(&state, user.id, &parent_deletion, now).await?;
    assert!(
        Task::get_owned(&mut db, parent.id, user.id)
            .await?
            .is_some()
    );
    let hidden = Task::get_by_id(&mut db, child.id).await?;
    assert_eq!(hidden.deletion_root_id, Some(child_deletion.id));
    assert!(hidden.deleted_at.is_some());
    restore(&state, user.id, &child_deletion, now).await?;
    assert!(Task::get_owned(&mut db, child.id, user.id).await?.is_some());
    Ok(())
}

#[tokio::test]
async fn restore_does_not_resume_a_series_stopped_before_deletion() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let (active, mut series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    let now = Timestamp::now();
    toasty::update!(series {
        stopped_at: Some(now),
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let deletion = remove(&state, user.id, active.id, now).await?;
    restore(&state, user.id, &deletion, now).await?;
    assert_eq!(
        TaskSeries::get_by_id(&mut db, series.id).await?.stopped_at,
        Some(now)
    );
    Ok(())
}

#[tokio::test]
async fn restores_a_subtree_with_completed_and_active_occurrences_in_one_series() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let parent = task(&mut db, user.id, None).await?;
    let (mut first, series) = recurring(&mut db, user.id, SeriesEnd::Never).await?;
    toasty::update!(first {
        parent_id: Some(parent.id)
    })
    .exec(&mut db)
    .await?;
    finish_at(&state, user.id, first.id, "2028-01-31T08:00:00Z").await?;
    let now = Timestamp::now();
    let deletion = remove(&state, user.id, parent.id, now).await?;
    restore(&state, user.id, &deletion, now).await?;
    let visible = Task::not_deleted()
        .filter(Task::fields().series_id().eq(Some(series.id)))
        .exec(&mut db)
        .await?;
    assert_eq!(visible.len(), 2);
    assert_eq!(
        visible
            .iter()
            .filter(|task| task.completed_at.is_some())
            .count(),
        1
    );
    assert!(
        TaskSeries::get_by_id(&mut db, series.id)
            .await?
            .stopped_at
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn soft_deleted_tasks_are_hidden_from_routes_lists_and_reminders() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let removed = task(&mut db, user.id, None).await?;
    let retained = task(&mut db, user.id, None).await?;
    let now = Timestamp::now();
    remove(&state, user.id, removed.id, now).await?;
    let start = removed.due_at.ok_or("missing due date")?;
    let before = now.checked_add(SignedDuration::from_secs(60))?;
    let reminders =
        Task::find_due_batch_after_task_id(&mut db, user.id, start, before, 0, 100).await?;
    assert_eq!(
        reminders.iter().map(|task| task.id).collect::<Vec<_>>(),
        vec![retained.id]
    );
    let token = create_session(&mut db, user.id, now).await?;
    let app = get_app_router(state.clone());
    for (method, suffix) in [
        ("GET", ""),
        ("PUT", ""),
        ("POST", "/complete"),
        ("DELETE", "/complete"),
    ] {
        let body = serde_json::json!({"name": "Edited", "tag_ids": []}).to_string();
        let request = Request::builder()
            .method(method)
            .uri(format!("/tasks/{}{suffix}", removed.id))
            .header("Cookie", format!("rincuma_session={}", token.expose()))
            .header("Content-Type", "application/json")
            .body(Body::from(body))?;
        assert_eq!(
            app.clone().oneshot(request).await?.status(),
            StatusCode::NOT_FOUND,
            "{method} {suffix}"
        );
    }
    for filter in ["all", "active", "upcoming", "completed", "overdue"] {
        let request = Request::builder()
            .uri(format!("/tasks?status={filter}"))
            .header("Cookie", format!("rincuma_session={}", token.expose()))
            .body(Body::empty())?;
        let response = app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        let value: serde_json::Value =
            serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 65536).await?)?;
        let items = value
            .get("items")
            .and_then(serde_json::Value::as_array)
            .ok_or("missing items")?;
        assert!(
            items
                .iter()
                .all(|item| item.get("id").and_then(serde_json::Value::as_i64) != Some(removed.id))
        );
        if filter == "all" {
            assert_eq!(
                value.get("total").and_then(serde_json::Value::as_u64),
                Some(1)
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn deleted_children_do_not_block_recurrence_or_project_edits() -> TestResult {
    use crate::web::{
        auth::CurrentUser,
        task::{handlers::update, relations::validate_recurrence_leaf, request::SaveTaskRequest},
    };
    use axum::{Json, extract::State};
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let parent = task(&mut db, user.id, None).await?;
    let child = task(&mut db, user.id, Some(parent.id)).await?;
    let now = Timestamp::now();
    remove(&state, user.id, child.id, now).await?;
    validate_recurrence_leaf(&mut db, parent.id).await?;
    let project = toasty::create!(Project {
        name: "Other project".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let request: SaveTaskRequest = serde_json::from_value(
        serde_json::json!({"name": parent.name, "due_at": parent.due_at, "project_id": project.id, "tag_ids": []}),
    )?;
    let Json(updated) = update(
        State(state),
        CurrentUser(user),
        axum::extract::Path(parent.id),
        Json(request),
    )
    .await?;
    assert_eq!(updated.project_id, Some(project.id));
    assert_eq!(
        Task::get_by_id(&mut db, parent.id).await?.project_id,
        Some(project.id)
    );
    assert_eq!(Task::get_by_id(&mut db, child.id).await?.project_id, None);
    Ok(())
}

#[tokio::test]
async fn stale_undo_cannot_restore_a_later_deletion_of_the_same_task() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let root = task(&mut db, user.id, None).await?;
    let now = Timestamp::now();
    let first = remove(&state, user.id, root.id, now).await?;
    restore(&state, user.id, &first, now).await?;
    let later = now.checked_add(SignedDuration::from_secs(1))?;
    let second = remove(&state, user.id, root.id, later).await?;
    assert_eq!(first.id, second.id);
    assert!(matches!(
        restore(&state, user.id, &first, later).await,
        Err(TaskError::Conflict(..))
    ));
    assert_eq!(
        Task::get_by_id(&mut db, root.id).await?.deleted_at,
        Some(second.deleted_at)
    );
    restore(&state, user.id, &second, later).await?;
    Ok(())
}

#[tokio::test]
async fn a_deleted_descendant_cannot_be_used_as_the_restoration_root() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let root = task(&mut db, user.id, None).await?;
    let child = task(&mut db, user.id, Some(root.id)).await?;
    let now = Timestamp::now();
    let deletion = remove(&state, user.id, root.id, now).await?;
    let partial = DeletionReceipt {
        id: child.id,
        deleted_at: deletion.deleted_at,
        expires_at: deletion.expires_at,
    };
    assert!(matches!(
        restore(&state, user.id, &partial, now).await,
        Err(TaskError::Conflict(..))
    ));
    assert!(Task::get_owned(&mut db, root.id, user.id).await?.is_none());
    assert!(Task::get_owned(&mut db, child.id, user.id).await?.is_none());
    restore(&state, user.id, &deletion, now).await?;
    Ok(())
}
