use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::get_project;
use crate::{
    db::{Project, User},
    web::{
        auth::CurrentUser, project::ProjectError, router::get_app_router,
        test_support::state_with_user,
    },
};

#[tokio::test]
async fn gets_owned_project_including_archived() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let project = toasty::create!(Project {
        name: "Archived project".to_owned(),
        description: Some("Details".to_owned()),
        color: Some("#AABBCC".to_owned()),
        archived_at: Some(now),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    let Json(response) = get_project(State(state), CurrentUser(user), Path(project.id)).await?;
    assert_eq!(response.id, project.id);
    assert_eq!(response.name, project.name);
    assert_eq!(response.description, project.description);
    assert_eq!(response.color, project.color);
    assert_eq!(response.archived_at, project.archived_at);
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
    let project = toasty::create!(Project {
        name: "Other user's project".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    for id in [project.id, project.id + 1] {
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let result = get_project(State(state.clone()), CurrentUser(current_user), Path(id)).await;
        assert!(matches!(
            result,
            Err(ProjectError::RelatedResourceNotFound(_))
        ));
    }
    Ok(())
}

#[tokio::test]
async fn get_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder().uri("/projects/1").body(Body::empty())?;
    let response = get_app_router(state).oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
