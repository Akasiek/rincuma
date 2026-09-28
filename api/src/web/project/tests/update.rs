use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::{SaveProjectRequest, update};
use crate::{
    db::{Project, User},
    web::{
        auth::CurrentUser, project::ProjectError, router::get_app_router,
        test_support::state_with_user,
    },
};

fn request(name: &str) -> SaveProjectRequest {
    SaveProjectRequest {
        name: name.to_owned(),
        description: Some("Updated description".to_owned()),
        color: Some("#AABBCC".to_owned()),
    }
}

async fn project(
    db: &mut toasty::Db,
    owner_id: i64,
) -> Result<Project, Box<dyn std::error::Error>> {
    let now: Timestamp = "2020-01-01T00:00:00Z".parse()?;
    Ok(toasty::create!(Project {
        name: "Original project".to_owned(),
        description: Some("Original description".to_owned()),
        color: Some("#112233".to_owned()),
        archived_at: Some(now),
        owner_id,
        created_at: now,
        updated_at: now,
    })
    .exec(db)
    .await?)
}

#[tokio::test]
async fn updates_owned_project_and_preserves_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = project(&mut db, user.id).await?;

    let Json(response) = update(
        State(state),
        CurrentUser(user),
        Path(original.id),
        Json(request("  Updated project  ")),
    )
    .await?;

    assert_eq!(response.id, original.id);
    assert_eq!(response.name, "Updated project");
    assert_eq!(response.description.as_deref(), Some("Updated description"));
    assert_eq!(response.color.as_deref(), Some("#AABBCC"));
    assert_eq!(response.owner_id, original.owner_id);
    assert_eq!(response.created_at, original.created_at);
    assert_eq!(response.archived_at, original.archived_at);
    assert!(response.updated_at > original.updated_at);

    let saved = Project::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, response.name);
    assert_eq!(saved.description, response.description);
    assert_eq!(saved.color, response.color);
    assert_eq!(saved.updated_at, response.updated_at);
    assert_eq!(saved.owner_id, original.owner_id);
    assert_eq!(saved.created_at, original.created_at);
    assert_eq!(saved.archived_at, original.archived_at);
    Ok(())
}

#[tokio::test]
async fn clears_optional_fields() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = project(&mut db, user.id).await?;
    let request = SaveProjectRequest {
        name: "Project".to_owned(),
        description: None,
        color: None,
    };

    let Json(response) = update(
        State(state),
        CurrentUser(user),
        Path(original.id),
        Json(request),
    )
    .await?;
    assert!(response.description.is_none());
    assert!(response.color.is_none());
    let saved = Project::get_by_id(&mut db, &original.id).await?;
    assert!(saved.description.is_none());
    assert!(saved.color.is_none());
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_input_without_changing_project() -> Result<(), Box<dyn std::error::Error>>
{
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let original = project(&mut db, user.id).await?;
    for request in [
        request("  "),
        request(&"x".repeat(256)),
        SaveProjectRequest {
            color: Some("red".to_owned()),
            ..request("Project")
        },
    ] {
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let result = update(
            State(state.clone()),
            CurrentUser(current_user),
            Path(original.id),
            Json(request),
        )
        .await;
        let error = result.err().ok_or("expected validation to fail")?;
        assert!(matches!(error, ProjectError::BadRequest(_)));
        assert_eq!(error.into_response().status(), StatusCode::BAD_REQUEST);
    }
    let saved = Project::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, original.name);
    assert_eq!(saved.description, original.description);
    assert_eq!(saved.color, original.color);
    assert_eq!(saved.updated_at, original.updated_at);
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_projects() -> Result<(), Box<dyn std::error::Error>> {
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
    let original = project(&mut db, other.id).await?;

    for id in [original.id, original.id + 1] {
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let result = update(
            State(state.clone()),
            CurrentUser(current_user),
            Path(id),
            Json(request("Updated project")),
        )
        .await;
        let error = result.err().ok_or("expected project lookup to fail")?;
        assert!(matches!(error, ProjectError::RelatedResourceNotFound(_)));
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
    let saved = Project::get_by_id(&mut db, &original.id).await?;
    assert_eq!(saved.name, original.name);
    assert_eq!(saved.description, original.description);
    assert_eq!(saved.color, original.color);
    assert_eq!(saved.updated_at, original.updated_at);
    Ok(())
}

#[tokio::test]
async fn update_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder()
        .method("PUT")
        .uri("/projects/1")
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"name":"Project","description":null,"color":null}"#,
        ))?;
    let response = get_app_router(state).oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
