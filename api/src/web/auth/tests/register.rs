#![allow(clippy::panic_in_result_fn)]

use std::path::PathBuf;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use super::{Credentials, register};
use crate::{
    app_state::AppState,
    config::AppConfig,
    db::User,
    web::test_support::{empty_state, state_with_user},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn credentials(email: &str) -> Json<Credentials> {
    Json(Credentials {
        email: email.to_owned(),
        password: "correct horse battery staple".to_owned(),
    })
}

fn config_with_registration() -> AppConfig {
    AppConfig::for_test(PathBuf::new()).with_registration_enabled()
}

async fn register_status(state: &AppState, email: &str) -> StatusCode {
    match register(State(state.clone()), credentials(email)).await {
        Ok(response) => response.into_response().status(),
        Err(error) => error.into_response().status(),
    }
}

async fn is_admin(state: &AppState, email: &str) -> toasty::Result<bool> {
    let mut db = state.db().clone();
    let user = User::get_by_email(&mut db, email.to_owned()).await?;
    Ok(user.is_admin)
}

#[tokio::test]
async fn disabled_registration_is_forbidden_even_for_empty_database() -> TestResult {
    let state = empty_state(AppConfig::for_test(PathBuf::new())).await?;

    let status = register_status(&state, "first@example.com").await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(!User::any_exist(&mut state.db().clone()).await?);
    Ok(())
}

#[tokio::test]
async fn first_registered_user_becomes_admin() -> TestResult {
    let state = empty_state(config_with_registration()).await?;

    assert_eq!(
        register_status(&state, "first@example.com").await,
        StatusCode::OK
    );
    assert_eq!(
        register_status(&state, "second@example.com").await,
        StatusCode::OK
    );

    assert!(is_admin(&state, "first@example.com").await?);
    assert!(!is_admin(&state, "second@example.com").await?);
    Ok(())
}

#[tokio::test]
async fn registration_into_non_empty_database_is_not_admin() -> TestResult {
    let (state, _) = state_with_user().await?;
    let state = AppState::new(config_with_registration(), state.db().clone());

    assert_eq!(
        register_status(&state, "new@example.com").await,
        StatusCode::OK
    );

    assert!(!is_admin(&state, "new@example.com").await?);
    Ok(())
}

#[tokio::test]
async fn response_includes_admin_flag() -> TestResult {
    let state = empty_state(config_with_registration()).await?;

    let response = register(State(state), credentials("first@example.com"))
        .await
        .map_err(|_| "register failed")?
        .into_response();
    let body = axum::body::to_bytes(response.into_body(), 1024).await?;
    let user: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(user.get("is_admin"), Some(&serde_json::Value::Bool(true)));
    Ok(())
}
