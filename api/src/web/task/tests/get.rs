use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::get_task;
use crate::{
    db::{Project, Tag, Task, TaskPriority, TaskTag, User},
    web::{
        auth::CurrentUser, router::get_app_router, task::TaskError, test_support::state_with_user,
    },
};

#[tokio::test]
async fn gets_owned_completed_task_with_relations() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let project = toasty::create!(Project {
        name: "Archived project".to_owned(),
        archived_at: Some(now),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let parent = toasty::create!(Task {
        name: "Parent".to_owned(),
        project_id: Some(project.id),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let task = toasty::create!(Task {
        name: "Completed task".to_owned(),
        description: Some("Details".to_owned()),
        due_at: Some(now),
        completed_at: Some(now),
        priority: TaskPriority::High,
        project_id: Some(project.id),
        parent_id: Some(parent.id),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let mut tag_ids = Vec::new();
    for name in ["First", "Second"] {
        let tag = toasty::create!(Tag {
            name: name.to_owned(),
            owner_id: user.id,
            created_at: now,
            updated_at: now,
        })
        .exec(&mut db)
        .await?;
        tag_ids.push(tag.id);
    }
    for tag_id in tag_ids.iter().rev() {
        toasty::create!(TaskTag {
            task_id: task.id,
            tag_id: *tag_id,
        })
        .exec(&mut db)
        .await?;
    }
    toasty::create!(TaskTag {
        task_id: parent.id,
        tag_id: *tag_ids.first().ok_or("missing test tag")?,
    })
    .exec(&mut db)
    .await?;

    let Json(response) = get_task(State(state), CurrentUser(user), Path(task.id)).await?;
    assert_eq!(response.id, task.id);
    assert_eq!(response.name, task.name);
    assert_eq!(response.description, task.description);
    assert_eq!(response.due_at, task.due_at);
    assert_eq!(response.completed_at, task.completed_at);
    assert_eq!(response.priority, task.priority);
    assert_eq!(response.owner_id, task.owner_id);
    assert_eq!(response.project_id, task.project_id);
    assert_eq!(response.parent_id, task.parent_id);
    assert_eq!(response.tag_ids, tag_ids);
    assert_eq!(response.created_at, task.created_at);
    assert_eq!(response.updated_at, task.updated_at);
    Ok(())
}

#[tokio::test]
async fn gets_owned_task_without_relations() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let task = toasty::create!(Task {
        name: "Task".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    let Json(response) = get_task(State(state), CurrentUser(user), Path(task.id)).await?;
    assert_eq!(response.id, task.id);
    assert!(response.description.is_none());
    assert!(response.due_at.is_none());
    assert!(response.completed_at.is_none());
    assert_eq!(response.priority, TaskPriority::None);
    assert!(response.project_id.is_none());
    assert!(response.parent_id.is_none());
    assert!(response.tag_ids.is_empty());
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let task = toasty::create!(Task {
        name: "Other user's task".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    for id in [task.id, task.id + 1] {
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let result = get_task(State(state.clone()), CurrentUser(current_user), Path(id)).await;
        let error = result.err().ok_or("expected task lookup to fail")?;
        assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
    Ok(())
}

#[tokio::test]
async fn get_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder().uri("/tasks/1").body(Body::empty())?;
    let response = get_app_router(state).oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
