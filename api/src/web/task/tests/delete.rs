use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::delete;
use crate::{
    auth::session::create_session,
    db::{Project, Tag, Task, TaskTag, User},
    web::{
        auth::CurrentUser, router::get_app_router, task::TaskError, test_support::state_with_user,
    },
};

async fn task(
    db: &mut toasty::Db,
    owner_id: i64,
    project_id: Option<i64>,
    parent_id: Option<i64>,
) -> toasty::Result<Task> {
    let now = Timestamp::now();
    toasty::create!(Task {
        name: "Task".to_owned(),
        owner_id,
        project_id,
        parent_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

#[tokio::test]
async fn soft_deletes_subtree_but_preserves_links_and_unrelated_records()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = Timestamp::now();
    let project = toasty::create!(Project {
        name: "Project".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let parent = task(&mut db, user.id, Some(project.id), None).await?;
    let removed = task(&mut db, user.id, Some(project.id), Some(parent.id)).await?;
    let child = task(&mut db, user.id, Some(project.id), Some(removed.id)).await?;
    let grandchild = task(&mut db, user.id, Some(project.id), Some(child.id)).await?;
    let second_child = task(&mut db, user.id, Some(project.id), Some(removed.id)).await?;
    let sibling = task(&mut db, user.id, Some(project.id), Some(parent.id)).await?;
    let tag = toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    for task_id in [
        removed.id,
        child.id,
        grandchild.id,
        second_child.id,
        parent.id,
    ] {
        toasty::create!(TaskTag {
            task_id,
            tag_id: tag.id
        })
        .exec(&mut db)
        .await?;
    }

    let Json(receipt) = delete(State(state), CurrentUser(user), Path(removed.id)).await?;
    assert_eq!(receipt.task_id, removed.id);
    for task_id in [removed.id, child.id, grandchild.id, second_child.id] {
        let deleted = Task::get_by_id(&mut db, task_id).await?;
        assert!(deleted.deleted_at.is_some());
        assert!(deleted.deletion_root_id.is_some());
        assert_eq!(
            TaskTag::filter_by_task_id(task_id)
                .exec(&mut db)
                .await?
                .len(),
            1
        );
    }
    for retained in [&parent, &sibling] {
        let saved = Task::get_by_id(&mut db, &retained.id).await?;
        assert_eq!(saved.parent_id, retained.parent_id);
        assert_eq!(saved.project_id, retained.project_id);
        assert_eq!(saved.updated_at, retained.updated_at);
    }
    assert_eq!(
        Project::get_by_id(&mut db, &project.id).await?.id,
        project.id
    );
    assert_eq!(Tag::get_by_id(&mut db, &tag.id).await?.id, tag.id);
    let links = TaskTag::filter_by_task_id(parent.id).exec(&mut db).await?;
    assert_eq!(links.len(), 1);
    assert_eq!(links.first().map(|link| link.tag_id), Some(tag.id));
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
    let retained = task(&mut db, other.id, None, None).await?;
    let child = task(&mut db, other.id, None, Some(retained.id)).await?;
    let tag = toasty::create!(Tag {
        name: "Other tag".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    toasty::create!(TaskTag {
        task_id: retained.id,
        tag_id: tag.id
    })
    .exec(&mut db)
    .await?;

    for id in [retained.id, child.id + 1] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let error = delete(State(state.clone()), CurrentUser(user), Path(id))
            .await
            .err()
            .ok_or("expected task lookup to fail")?;
        assert!(matches!(error, TaskError::RelatedResourceNotFound(_)));
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
    assert_eq!(
        Task::get_by_id(&mut db, &retained.id).await?.id,
        retained.id
    );
    assert_eq!(
        Task::get_by_id(&mut db, &child.id).await?.parent_id,
        Some(retained.id)
    );
    assert_eq!(
        TaskTag::filter_by_task_id(retained.id)
            .exec(&mut db)
            .await?
            .len(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn delete_route_authenticates_and_returns_undo_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let task = task(&mut db, user.id, None, None).await?;
    let token = create_session(&mut db, user.id, Timestamp::now()).await?;
    let app = get_app_router(state);
    for (authenticated, status) in [
        (false, StatusCode::UNAUTHORIZED),
        (true, StatusCode::OK),
        (true, StatusCode::NOT_FOUND),
    ] {
        let mut request = Request::builder()
            .method("DELETE")
            .uri(format!("/tasks/{}", task.id));
        if authenticated {
            request = request.header("Cookie", format!("rincuma_session={}", token.expose()));
        }
        let response = app.clone().oneshot(request.body(Body::empty())?).await?;
        assert_eq!(response.status(), status);
        let body = axum::body::to_bytes(response.into_body(), 4096).await?;
        if status == StatusCode::OK {
            let value: serde_json::Value = serde_json::from_slice(&body)?;
            assert_eq!(
                value.get("task_id").and_then(serde_json::Value::as_i64),
                Some(task.id)
            );
            assert!(value.get("deleted_at").is_some());
            assert!(value.get("expires_at").is_some());
        } else {
            assert!(body.is_empty());
        }
    }
    Ok(())
}
