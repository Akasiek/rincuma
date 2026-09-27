use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;

use super::{CreateProjectRequest, create};
use crate::{
    db::Project,
    web::{auth::CurrentUser, project::ProjectError, test_support::state_with_user},
};

#[tokio::test]
async fn creates_project_for_current_user() -> Result<(), Box<dyn std::error::Error>> {
    let (state, user) = state_with_user().await?;
    let owner_id = user.id;
    let request = CreateProjectRequest {
        name: "  Example project  ".to_owned(),
        description: Some("Description".to_owned()),
        color: Some("#3B82F6".to_owned()),
    };

    let (status, Json(response)) =
        create(State(state.clone()), CurrentUser(user), Json(request)).await?;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response.name, "Example project");
    assert_eq!(response.description.as_deref(), Some("Description"));
    assert_eq!(response.color.as_deref(), Some("#3B82F6"));
    assert_eq!(response.owner_id, owner_id);
    assert!(response.archived_at.is_none());
    assert_eq!(response.created_at, response.updated_at);

    let mut db = state.db().clone();
    let saved = Project::get_by_id(&mut db, &response.id).await?;
    assert_eq!(saved.name, response.name);
    assert_eq!(saved.owner_id, owner_id);
    assert_eq!(saved.color, response.color);
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_project_names_and_colors() -> Result<(), Box<dyn std::error::Error>> {
    for (name, color) in [
        ("  ", None),
        (&"x".repeat(256), None),
        ("Project", Some("red")),
    ] {
        let (state, user) = state_with_user().await?;
        let request = CreateProjectRequest {
            name: name.to_owned(),
            description: None,
            color: color.map(str::to_owned),
        };

        let result = create(State(state), CurrentUser(user), Json(request)).await;
        assert!(matches!(result, Err(ProjectError::BadRequest(_))));
    }
    Ok(())
}

#[test]
fn validates_optional_hex_color() {
    for color in [None, Some("#3B82F6"), Some("#abcdef")] {
        let request = CreateProjectRequest {
            name: "Project".to_owned(),
            description: None,
            color: color.map(str::to_owned),
        };
        assert!(request.validate().is_ok());
    }

    for color in ["#FFF", "3B82F6", "#3B82FG", "#3B82F6FF", ""] {
        let request = CreateProjectRequest {
            name: "Project".to_owned(),
            description: None,
            color: Some(color.to_owned()),
        };
        assert!(request.validate().is_err());
    }
}
