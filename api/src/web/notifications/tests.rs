#![allow(clippy::panic_in_result_fn)]

use super::{SaveSettings, delete_settings, get_settings, save_settings};
use crate::{
    db::{NotificationProviderKind, User},
    web::{auth::CurrentUser, test_support::state_with_user},
};
use axum::{Json, extract::State, http::StatusCode};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn request(url: &str, authorization: Option<&str>) -> SaveSettings {
    SaveSettings {
        provider: NotificationProviderKind::Ntfy,
        enabled: true,
        url: url.to_owned(),
        authorization: authorization.map(str::to_owned),
    }
}

#[tokio::test]
async fn settings_are_scoped_and_credentials_are_preserved_or_cleared() -> TestResult {
    let (state, user) = state_with_user().await?;
    let user_id = user.id;
    let mut db = state.db().clone();
    let Json(initial) = get_settings(State(state.clone()), CurrentUser(user))
        .await
        .map_err(|_| "get failed")?;
    assert!(initial.is_none());
    for auth in [Some("Bearer secret"), None, Some("")] {
        let user = User::get_by_id(&mut db, &user_id).await?;
        let Json(saved) = save_settings(
            State(state.clone()),
            CurrentUser(user),
            Json(request("https://ntfy.example/tasks", auth)),
        )
        .await
        .map_err(|_| "save failed")?;
        assert_eq!(saved.has_authorization, auth != Some(""));
        assert_eq!(saved.provider, "ntfy");
    }
    let now = jiff::Timestamp::now();
    let other = toasty::create!(User {
        email: "other@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now
    })
    .exec(&mut db)
    .await?;
    let Json(unconfigured) = get_settings(State(state.clone()), CurrentUser(other))
        .await
        .map_err(|_| "get failed")?;
    assert!(unconfigured.is_none());
    let user = User::get_by_id(&mut db, &user_id).await?;
    assert_eq!(
        delete_settings(State(state.clone()), CurrentUser(user)).await,
        Ok(StatusCode::NO_CONTENT)
    );
    Ok(())
}

#[test]
fn rejects_invalid_destinations_and_header_injection() {
    for url in [
        "file:///tmp/topic",
        "https://ntfy.example",
        "https://user:password@ntfy.example/tasks",
        "https://ntfy.example/tasks?auth=secret",
        "https://ntfy.example/tasks#fragment",
    ] {
        assert_eq!(
            super::validate_settings(&request(url, None)),
            Err(StatusCode::BAD_REQUEST)
        );
    }
    assert_eq!(
        super::validate_settings(&request(
            "http://localhost:8080/tasks",
            Some("Bearer secret\r\nX-Injected: true")
        )),
        Err(StatusCode::BAD_REQUEST)
    );
    assert!(
        super::validate_settings(&request(
            "http://localhost:8080/tasks",
            Some("Basic dXNlcjpwYXNz")
        ))
        .is_ok()
    );
}

#[tokio::test]
async fn routes_require_authentication() -> TestResult {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let (state, _) = state_with_user().await?;
    for method in ["GET", "PUT", "DELETE"] {
        let response = crate::web::router::get_app_router(state.clone())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/notifications/settings")
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

#[tokio::test]
async fn rejects_unsupported_provider_without_saving_settings() -> TestResult {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let token =
        crate::auth::session::create_session(&mut db, user.id, jiff::Timestamp::now()).await?;
    let response = crate::web::router::get_app_router(state.clone())
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/notifications/settings")
                .header("Content-Type", "application/json")
                .header("Cookie", format!("rincuma_session={}", token.expose()))
                .body(Body::from(
                    r#"{"provider":"gotify","enabled":true,"url":"https://gotify.example/tasks"}"#,
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        crate::db::NotificationSettings::filter_by_owner_id(user.id)
            .first()
            .exec(&mut db)
            .await?
            .is_none()
    );
    Ok(())
}
