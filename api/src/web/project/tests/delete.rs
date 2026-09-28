use axum::{
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
        auth::CurrentUser, project::ProjectError, router::get_app_router,
        test_support::state_with_user,
    },
};

async fn project(db: &mut toasty::Db, owner_id: i64, archived: bool) -> toasty::Result<Project> {
    let now = Timestamp::now();
    toasty::create!(Project {
        name: "Project".to_owned(),
        archived_at: archived.then_some(now),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

#[tokio::test]
async fn deletes_project_but_preserves_tasks_hierarchy_and_tags()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let removed = project(&mut db, user.id, false).await?;
    let retained = project(&mut db, user.id, false).await?;
    let now: Timestamp = "2020-01-01T00:00:00Z".parse()?;
    let parent = toasty::create!(Task {
        name: "Parent".to_owned(),
        project_id: Some(removed.id),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let child = toasty::create!(Task {
        name: "Child".to_owned(),
        project_id: Some(removed.id),
        parent_id: Some(parent.id),
        completed_at: Some(now),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let unrelated = toasty::create!(Task {
        name: "Unrelated".to_owned(),
        project_id: Some(retained.id),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let tag = toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    toasty::create!(TaskTag {
        task_id: child.id,
        tag_id: tag.id
    })
    .exec(&mut db)
    .await?;

    let status = delete(State(state), CurrentUser(user), Path(removed.id)).await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(
        Project::filter_by_id(removed.id)
            .first()
            .exec(&mut db)
            .await?
            .is_none()
    );
    for original in [&parent, &child] {
        let saved = Task::get_by_id(&mut db, &original.id).await?;
        assert!(saved.project_id.is_none());
        assert_eq!(saved.name, original.name);
        assert_eq!(saved.parent_id, original.parent_id);
        assert_eq!(saved.completed_at, original.completed_at);
        assert_eq!(saved.owner_id, original.owner_id);
        assert_eq!(saved.created_at, original.created_at);
        assert_eq!(saved.updated_at, original.updated_at);
    }
    let parent = Task::get_by_id(&mut db, &parent.id).await?;
    let child = Task::get_by_id(&mut db, &child.id).await?;
    assert_eq!(parent.updated_at, child.updated_at);
    let unrelated_saved = Task::get_by_id(&mut db, &unrelated.id).await?;
    assert_eq!(unrelated_saved.project_id, unrelated.project_id);
    assert_eq!(unrelated_saved.updated_at, unrelated.updated_at);
    assert_eq!(
        Project::get_by_id(&mut db, &retained.id).await?.id,
        retained.id
    );
    assert_eq!(Tag::get_by_id(&mut db, &tag.id).await?.id, tag.id);
    let links = TaskTag::filter_by_task_id(child.id).exec(&mut db).await?;
    assert_eq!(links.len(), 1);
    assert_eq!(links.first().map(|link| link.tag_id), Some(tag.id));
    Ok(())
}

#[tokio::test]
async fn deletes_empty_archived_project() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let project = project(&mut db, user.id, true).await?;
    assert_eq!(
        delete(State(state), CurrentUser(user), Path(project.id)).await?,
        StatusCode::NO_CONTENT
    );
    assert!(
        Project::filter_by_id(project.id)
            .first()
            .exec(&mut db)
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_projects() -> Result<(), Box<dyn std::error::Error>> {
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
    let retained = project(&mut db, other.id, false).await?;
    let task = toasty::create!(Task {
        name: "Other task".to_owned(),
        project_id: Some(retained.id),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    for id in [retained.id, retained.id + 1] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let error = delete(State(state.clone()), CurrentUser(user), Path(id))
            .await
            .err()
            .ok_or("expected project lookup to fail")?;
        assert!(matches!(error, ProjectError::RelatedResourceNotFound(_)));
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
    assert_eq!(
        Project::get_by_id(&mut db, &retained.id).await?.id,
        retained.id
    );
    let saved = Task::get_by_id(&mut db, &task.id).await?;
    assert_eq!(saved.project_id, task.project_id);
    assert_eq!(saved.updated_at, task.updated_at);
    Ok(())
}

#[tokio::test]
async fn delete_route_authenticates_and_returns_empty_response()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let project = project(&mut db, user.id, false).await?;
    let token = create_session(&mut db, user.id, Timestamp::now()).await?;
    let app = get_app_router(state);
    for (authenticated, status) in [
        (false, StatusCode::UNAUTHORIZED),
        (true, StatusCode::NO_CONTENT),
        (true, StatusCode::NOT_FOUND),
    ] {
        let mut request = Request::builder()
            .method("DELETE")
            .uri(format!("/projects/{}", project.id));
        if authenticated {
            request = request.header("Cookie", format!("rincuma_session={}", token.expose()));
        }
        let response = app.clone().oneshot(request.body(Body::empty())?).await?;
        assert_eq!(response.status(), status);
        let body = axum::body::to_bytes(response.into_body(), 1024).await?;
        assert!(body.is_empty());
    }
    Ok(())
}
