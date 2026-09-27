use axum::{Json, extract::State, http::StatusCode};

use super::{CreateTagRequest, create};
use crate::{
    db::Tag,
    web::{auth::CurrentUser, tag::TagError, test_support::state_with_user},
};

#[tokio::test]
async fn creates_tag_for_current_user() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let owner_id = user.id;
    let request = CreateTagRequest {
        name: "  Example tag  ".to_owned(),
        color: Some("#3B82F6".to_owned()),
    };

    let (status, Json(response)) =
        create(State(state.clone()), CurrentUser(user), Json(request)).await?;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response.name, "Example tag");
    assert_eq!(response.color.as_deref(), Some("#3B82F6"));
    assert_eq!(response.owner_id, owner_id);
    assert_eq!(response.created_at, response.updated_at);

    let mut db = state.db().clone();
    let saved = Tag::get_by_id(&mut db, &response.id).await?;
    assert_eq!(saved.name, response.name);
    assert_eq!(saved.color, response.color);
    assert_eq!(saved.owner_id, owner_id);
    Ok(())
}

#[tokio::test]
async fn creates_tag_without_color() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let request = CreateTagRequest {
        name: "Tag".to_owned(),
        color: None,
    };

    let (status, Json(response)) = create(State(state), CurrentUser(user), Json(request)).await?;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response.color.is_none());
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_tag_names_and_colors() -> Result<(), Box<dyn std::error::Error>> {
    for (name, color) in [
        ("  ", None),
        (&"x".repeat(256), None),
        ("Tag", Some("red")),
        ("Tag", Some("#12345G")),
    ] {
        let (state, user) = state_with_user().await?;
        let request = CreateTagRequest {
            name: name.to_owned(),
            color: color.map(str::to_owned),
        };

        let result = create(State(state), CurrentUser(user), Json(request)).await;
        assert!(matches!(result, Err(TagError::BadRequest(_))));
    }
    Ok(())
}
