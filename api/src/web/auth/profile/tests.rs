#![allow(clippy::panic_in_result_fn)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::{
    app_state::AppState,
    auth::{
        password::{hash_password, verify_password},
        session::create_session,
    },
    db::{Session, User},
    web::{router::get_app_router, test_support::state_with_user},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn put(
    state: AppState,
    path: &str,
    token: &str,
    body: &str,
) -> Result<StatusCode, Box<dyn std::error::Error>> {
    let response = get_app_router(state)
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(path)
                .header("Content-Type", "application/json")
                .header("Cookie", format!("rincuma_session={token}"))
                .body(Body::from(body.to_owned()))?,
        )
        .await?;
    Ok(response.status())
}

#[tokio::test]
async fn email_change_is_normalized_and_revokes_only_other_sessions() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = jiff::Timestamp::now();
    let current = create_session(&mut db, user.id, now).await?;
    let other = create_session(&mut db, user.id, now).await?;
    let stranger = toasty::create!(User {
        email: "stranger@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    let stranger_session = create_session(&mut db, stranger.id, now).await?;

    assert_eq!(
        put(
            state.clone(),
            "/auth/me",
            current.expose(),
            r#"{"email":" NEW@Example.com "}"#
        )
        .await?,
        StatusCode::OK
    );
    let updated = User::get_by_id(&mut db, &user.id).await?;
    assert_eq!(updated.email, "new@example.com");
    assert_eq!(updated.password_hash, user.password_hash);
    assert!(Session::is_valid_for_user(&mut db, &current.digest(), user.id, now).await?);
    assert!(!Session::is_valid_for_user(&mut db, &other.digest(), user.id, now).await?);
    assert!(
        Session::is_valid_for_user(&mut db, &stranger_session.digest(), stranger.id, now).await?
    );
    assert_eq!(
        put(
            state,
            "/auth/me",
            other.expose(),
            r#"{"email":"blocked@example.com"}"#
        )
        .await?,
        StatusCode::UNAUTHORIZED
    );
    Ok(())
}

#[tokio::test]
async fn rejected_email_changes_keep_profile_and_sessions() -> TestResult {
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let now = jiff::Timestamp::now();
    let current = create_session(&mut db, user.id, now).await?;
    let other = create_session(&mut db, user.id, now).await?;
    toasty::create!(User {
        email: "taken@example.com".to_owned(),
        password_hash: "unused".to_owned(),
        created_at: now,
        updated_at: now,
    })
    .exec(&mut db)
    .await?;
    for (body, expected) in [
        (r#"{"email":"taken@example.com"}"#, StatusCode::CONFLICT),
        (r#"{"email":"not an email"}"#, StatusCode::BAD_REQUEST),
    ] {
        assert_eq!(
            put(state.clone(), "/auth/me", current.expose(), body).await?,
            expected
        );
        assert_eq!(User::get_by_id(&mut db, &user.id).await?.email, user.email);
        assert!(Session::is_valid_for_user(&mut db, &other.digest(), user.id, now).await?);
    }
    Ok(())
}

#[tokio::test]
async fn password_change_validates_current_and_confirmation_before_revoking_sessions() -> TestResult
{
    let (state, mut user) = state_with_user().await?;
    let mut db = state.db().clone();
    let hash = hash_password("old password".to_owned()).await?;
    toasty::update!(user {
        password_hash: hash
    })
    .exec(&mut db)
    .await?;
    let now = jiff::Timestamp::now();
    let current = create_session(&mut db, user.id, now).await?;
    let other = create_session(&mut db, user.id, now).await?;
    for (body, expected) in [
        (
            r#"{"current_password":"wrong","new_password":"new password","password_confirmation":"new password"}"#,
            StatusCode::FORBIDDEN,
        ),
        (
            r#"{"current_password":"old password","new_password":"new password","password_confirmation":"different"}"#,
            StatusCode::BAD_REQUEST,
        ),
        (
            r#"{"current_password":"old password","new_password":"","password_confirmation":""}"#,
            StatusCode::BAD_REQUEST,
        ),
    ] {
        assert_eq!(
            put(state.clone(), "/auth/password", current.expose(), body).await?,
            expected
        );
        let unchanged = User::get_by_id(&mut db, &user.id).await?;
        assert_eq!(unchanged.password_hash, user.password_hash);
        assert!(Session::is_valid_for_user(&mut db, &other.digest(), user.id, now).await?);
    }
    assert_eq!(put(state.clone(), "/auth/password", current.expose(), r#"{"current_password":"old password","new_password":"new password","password_confirmation":"new password"}"#).await?, StatusCode::NO_CONTENT);
    let updated = User::get_by_id(&mut db, &user.id).await?;
    assert!(verify_password("new password".to_owned(), updated.password_hash.clone()).await?);
    assert!(!verify_password("old password".to_owned(), updated.password_hash).await?);
    assert_eq!(updated.email, user.email);
    assert!(Session::is_valid_for_user(&mut db, &current.digest(), user.id, now).await?);
    assert!(!Session::is_valid_for_user(&mut db, &other.digest(), user.id, now).await?);
    Ok(())
}

#[tokio::test]
async fn profile_routes_require_a_valid_session() -> TestResult {
    let (state, _) = state_with_user().await?;
    for (path, body) in [
        ("/auth/me", r#"{"email":"new@example.com"}"#),
        (
            "/auth/password",
            r#"{"current_password":"old","new_password":"new","password_confirmation":"new"}"#,
        ),
    ] {
        assert_eq!(
            put(state.clone(), path, "invalid", body).await?,
            StatusCode::UNAUTHORIZED
        );
    }
    Ok(())
}

#[tokio::test]
async fn stale_profile_changes_cannot_overwrite_a_newer_change() -> TestResult {
    use super::service::save_profile;
    let (state, user) = state_with_user().await?;
    let mut db = state.db().clone();
    let stale_user = User::get_by_id(&mut db, &user.id).await?;
    let token = create_session(&mut db, user.id, jiff::Timestamp::now()).await?;
    save_profile(&state, &user, &token, "new@example.com").await?;
    let result = save_profile(&state, &stale_user, &token, "stale@example.com").await;
    assert!(matches!(
        result,
        Err(crate::web::auth::error::AuthError::StaleProfile)
    ));
    assert_eq!(
        User::filter_by_email("new@example.com")
            .first()
            .exec(&mut db)
            .await?
            .map(|user| user.email),
        Some("new@example.com".to_owned())
    );
    Ok(())
}
