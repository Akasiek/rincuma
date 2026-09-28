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
    db::{Tag, Task, TaskTag, User},
    web::{
        auth::CurrentUser, router::get_app_router, tag::TagError, test_support::state_with_user,
    },
};

async fn tag(db: &mut toasty::Db, owner_id: i64) -> toasty::Result<Tag> {
    let now = Timestamp::now();
    toasty::create!(Tag {
        name: "Tag".to_owned(),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await
}

#[tokio::test]
async fn deletes_tag_and_its_links_but_preserves_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let removed = tag(&mut db, user.id).await?;
    let retained = tag(&mut db, user.id).await?;
    let now = Timestamp::now();
    let task = toasty::create!(Task {
        name: "Task".to_owned(),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    for tag_id in [removed.id, retained.id] {
        toasty::create!(TaskTag {
            task_id: task.id,
            tag_id
        })
        .exec(&mut db)
        .await?;
    }

    let status = delete(State(state.clone()), CurrentUser(user), Path(removed.id)).await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(
        Tag::filter_by_id(removed.id)
            .first()
            .exec(&mut db)
            .await?
            .is_none()
    );
    assert!(
        TaskTag::filter_by_tag_id(removed.id)
            .exec(&mut db)
            .await?
            .is_empty()
    );
    let saved = Task::get_by_id(&mut db, &task.id).await?;
    assert_eq!(saved.name, task.name);
    assert_eq!(saved.updated_at, task.updated_at);
    assert_eq!(Tag::get_by_id(&mut db, &retained.id).await?.id, retained.id);
    let links = TaskTag::filter_by_task_id(task.id).exec(&mut db).await?;
    assert_eq!(links.len(), 1);
    assert_eq!(links.first().map(|link| link.tag_id), Some(retained.id));
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_tags() -> Result<(), Box<dyn std::error::Error>> {
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
    let retained = tag(&mut db, other.id).await?;
    let task = toasty::create!(Task {
        name: "Other task".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    toasty::create!(TaskTag {
        task_id: task.id,
        tag_id: retained.id
    })
    .exec(&mut db)
    .await?;
    for id in [retained.id, retained.id + 1] {
        let user = User::get_by_id(&mut db, &user.id).await?;
        let error = delete(State(state.clone()), CurrentUser(user), Path(id))
            .await
            .err()
            .ok_or("expected tag lookup to fail")?;
        assert!(matches!(error, TagError::RelatedResourceNotFound(_)));
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
    assert_eq!(Tag::get_by_id(&mut db, &retained.id).await?.id, retained.id);
    assert_eq!(
        TaskTag::filter_by_tag_id(retained.id)
            .exec(&mut db)
            .await?
            .len(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn delete_route_authenticates_and_returns_empty_response()
-> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let tag = tag(&mut db, user.id).await?;
    let token = create_session(&mut db, user.id, Timestamp::now()).await?;
    let app = get_app_router(state);
    for (authenticated, status) in [
        (false, StatusCode::UNAUTHORIZED),
        (true, StatusCode::NO_CONTENT),
        (true, StatusCode::NOT_FOUND),
    ] {
        let mut request = Request::builder()
            .method("DELETE")
            .uri(format!("/tags/{}", tag.id));
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
