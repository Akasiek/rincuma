use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
};
use jiff::Timestamp;
use tower::ServiceExt;

use super::get_tag;
use crate::{
    db::{Tag, User},
    web::{
        auth::CurrentUser, router::get_app_router, tag::TagError, test_support::state_with_user,
    },
};

#[tokio::test]
async fn gets_owned_tag() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let now = Timestamp::now();
    let mut db = state.db().clone();
    let tag = toasty::create!(Tag {
        name: "Important".to_owned(),
        color: Some("#AABBCC".to_owned()),
        owner_id: user.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    let Json(response) = get_tag(State(state), CurrentUser(user), Path(tag.id)).await?;
    assert_eq!(response.id, tag.id);
    assert_eq!(response.name, tag.name);
    assert_eq!(response.color, tag.color);
    assert_eq!(response.owner_id, tag.owner_id);
    Ok(())
}

#[tokio::test]
async fn hides_missing_and_unowned_tags() -> Result<(), Box<dyn std::error::Error>> {
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
    let tag = toasty::create!(Tag {
        name: "Other user's tag".to_owned(),
        owner_id: other.id,
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;

    for id in [tag.id, tag.id + 1] {
        let current_user = User::get_by_id(&mut db, &user.id).await?;
        let result = get_tag(State(state.clone()), CurrentUser(current_user), Path(id)).await;
        assert!(matches!(result, Err(TagError::RelatedResourceNotFound(_))));
    }
    Ok(())
}

#[tokio::test]
async fn get_route_requires_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let (state, _) = state_with_user().await?;
    let request = Request::builder().uri("/tags/1").body(Body::empty())?;
    let response = get_app_router(state).oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
