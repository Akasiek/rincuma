use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::{complete, reopen};
use crate::{
    auth::session::create_session,
    db::{Tag, Task, TaskPriority, TaskTag, User},
    web::{
        auth::CurrentUser, router::get_app_router, task::TaskError, test_support::state_with_user,
    },
};

async fn task(db: &mut toasty::Db, owner_id: i64) -> toasty::Result<Task> {
    let now = Timestamp::now();
    toasty::create!(Task {
        name: "Task".to_owned(),
        description: Some("Keep description".to_owned()),
        due_at: Some(now),
        priority: TaskPriority::High,
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

#[tokio::test]
async fn completes_only_selected_task_and_preserves_fields_and_tags()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = task(&mut db, user.id).await?;
    let mut child = task(&mut db, user.id).await?;
    toasty::update!(child {
        parent_id: Some(original.id)
    })
    .exec(&mut db)
    .await?;
    let now = Timestamp::now();
    let tag = toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id: user.id,
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

    let Json(response) =
        complete(State(state.clone()), CurrentUser(user), Path(original.id)).await?;
    let saved = Task::get_by_id(&mut db, &original.id).await?;
    assert!(saved.completed_at.is_some());
    assert_eq!(saved.completed_at, Some(saved.updated_at));
    assert_eq!(response.completed_at, saved.completed_at);
    assert_eq!(response.tag_ids, vec![tag.id]);
    assert_eq!(saved.name, original.name);
    assert_eq!(saved.description, original.description);
    assert_eq!(saved.due_at, original.due_at);
    assert_eq!(saved.priority, original.priority);
    assert_eq!(saved.project_id, original.project_id);
    assert_eq!(saved.parent_id, original.parent_id);
    assert_eq!(saved.created_at, original.created_at);
    assert!(
        Task::get_by_id(&mut db, &child.id)
            .await?
            .completed_at
            .is_none()
    );
    assert_eq!(
        TaskTag::filter_by_task_id(original.id)
            .exec(&mut db)
            .await?
            .len(),
        1
    );

    let user = User::get_by_id(&mut db, &original.owner_id).await?;
    let _ = complete(State(state.clone()), CurrentUser(user), Path(original.id)).await?;
    let repeated = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(repeated.completed_at, saved.completed_at);
    assert_eq!(repeated.updated_at, saved.updated_at);

    let user = User::get_by_id(&mut db, &original.owner_id).await?;
    let Json(response) = reopen(State(state.clone()), CurrentUser(user), Path(original.id)).await?;
    let reopened = Task::get_by_id(&mut db, &original.id).await?;
    assert!(response.completed_at.is_none());
    assert!(reopened.completed_at.is_none());
    assert_eq!(response.tag_ids, vec![tag.id]);
    assert_eq!(reopened.name, original.name);
    assert_eq!(reopened.description, original.description);
    assert_eq!(reopened.due_at, original.due_at);
    assert_eq!(reopened.priority, original.priority);
    assert_eq!(reopened.created_at, original.created_at);

    let user = User::get_by_id(&mut db, &original.owner_id).await?;
    let _ = reopen(State(state), CurrentUser(user), Path(original.id)).await?;
    let repeated = Task::get_by_id(&mut db, &original.id).await?;
    assert_eq!(repeated.updated_at, reopened.updated_at);
    assert!(repeated.completed_at.is_none());
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let unowned = task(&mut db, other.id).await?;
    for id in [unowned.id, unowned.id + 1] {
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let error = complete(State(state.clone()), CurrentUser(current_user), Path(id))
            .await
            .err()
            .ok_or("expected task lookup to fail")?;
        assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let error = reopen(State(state.clone()), CurrentUser(current_user), Path(id))
            .await
            .err()
            .ok_or("expected task lookup to fail")?;
        assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
    }
    assert!(
        Task::get_by_id(&mut db, &unowned.id)
            .await?
            .completed_at
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn complete_route_authenticates_and_returns_completed_task()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = task(&mut db, user.id).await?;
    let token = create_session(&mut db, user.id, Timestamp::now()).await?;
    let app = get_app_router(state);
    for (method, authenticated, status) in [
        ("POST", false, StatusCode::UNAUTHORIZED),
        ("POST", true, StatusCode::OK),
        ("DELETE", false, StatusCode::UNAUTHORIZED),
        ("DELETE", true, StatusCode::OK),
        ("DELETE", true, StatusCode::OK),
    ] {
        let mut request = Request::builder()
            .method(method)
            .uri(format!("/tasks/{}/complete", original.id));
        if authenticated {
            request = request.header("Cookie", format!("rincuma_session={}", token.expose()));
        }
        let response = app.clone().oneshot(request.body(Body::empty())?).await?;
        assert_eq!(response.status(), status);
        if authenticated {
            let body = axum::body::to_bytes(response.into_body(), 4096).await?;
            let body = std::str::from_utf8(&body)?;
            assert!(body.contains(&format!("\"id\":{},", original.id)));
            if method == "POST" {
                assert!(body.contains("\"completed_at\":\""));
            } else {
                assert!(body.contains("\"completed_at\":null"));
            }
        }
    }
    Ok(())
}
