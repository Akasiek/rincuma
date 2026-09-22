use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    auth::{
        error::AuthError,
        extractor::CurrentUser,
        password::{PasswordError, hash_password, verify_password},
        session::create_session,
        session_cookie::{removal_cookie, session_cookie, session_token},
        time::current_unix_timestamp_seconds,
        token::SessionToken,
    },
    db::{Session, User},
};

#[derive(Deserialize)]
pub(super) struct Credentials {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub(super) struct UserResponse {
    id: i64,
    email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
        }
    }
}

pub(super) async fn register(
    state: axum::extract::State<AppState>,
    Json(credentials): Json<Credentials>,
) -> Result<impl IntoResponse, AuthError> {
    let email = normalize_email(&credentials.email)?;
    let password_hash = hash_password(credentials.password)
        .await
        .map_err(map_password_error)?;
    let now = current_unix_timestamp_seconds()?;
    let mut db = state.db().clone();

    let mut transaction = db.transaction().await.map_err(AuthError::from)?;
    let user = User::upsert_by_email(email)
        .password_hash(password_hash)
        .created_at(now)
        .updated_at(now)
        .or_ignore()
        .exec(&mut transaction)
        .await
        .map_err(AuthError::from)?
        .ok_or(AuthError::Conflict)?;
    let token = create_session(&mut transaction, user.id, now).await?;
    transaction.commit().await.map_err(AuthError::from)?;

    authenticated_response(&state, &token, user)
}

pub(super) async fn login(
    state: axum::extract::State<AppState>,
    Json(credentials): Json<Credentials>,
) -> Result<impl IntoResponse, AuthError> {
    let email = normalize_email(&credentials.email)?;
    let mut db = state.db().clone();
    let user = User::filter_by_email(email)
        .first()
        .exec(&mut db)
        .await
        .map_err(AuthError::from)?;

    let Some(user) = user else {
        // Keep the failure path expensive to reduce timing-based account enumeration.
        hash_password(credentials.password)
            .await
            .map_err(map_password_error)?;
        return Err(AuthError::Unauthorized);
    };

    let password_valid = verify_password(credentials.password, user.password_hash.clone())
        .await
        .map_err(map_password_error)?;

    if !user.is_active || !password_valid {
        return Err(AuthError::Unauthorized);
    }

    let now = current_unix_timestamp_seconds()?;
    let token = create_session(&mut db, user.id, now).await?;
    authenticated_response(&state, &token, user)
}

pub(super) async fn logout(
    state: axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AuthError> {
    if let Some(token) = session_token(&headers, state.config().cookie_secure()) {
        let mut db = state.db().clone();
        Session::filter_by_token_hash(token.digest())
            .delete()
            .exec(&mut db)
            .await
            .map_err(AuthError::from)?;
    }

    let cookie = removal_cookie(state.config().cookie_secure());
    let header_value = HeaderValue::from_str(&cookie.to_string()).map_err(|_| AuthError::Internal)?;

    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, header_value)]))
}

pub(super) async fn me(CurrentUser(user): CurrentUser) -> Json<UserResponse> {
    Json(user.into())
}

fn authenticated_response(
    state: &AppState,
    token: &SessionToken,
    user: User,
) -> Result<Response, AuthError> {
    let cookie = session_cookie(token.expose(), state.config().cookie_secure());
    let header_value =
        HeaderValue::from_str(&cookie.to_string()).map_err(|_| AuthError::Internal)?;

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, header_value)],
        Json(UserResponse::from(user)),
    )
        .into_response())
}

fn normalize_email(email: &str) -> Result<String, AuthError> {
    let email = email.trim().to_lowercase();
    let Some((local, domain)) = email.split_once('@') else {
        return Err(AuthError::BadRequest);
    };

    if email.len() > 254
        || local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || email.chars().any(char::is_whitespace)
    {
        return Err(AuthError::BadRequest);
    }

    Ok(email)
}

#[allow(clippy::needless_pass_by_value)]
fn map_password_error(error: PasswordError) -> AuthError {
    match error {
        PasswordError::TooLong => AuthError::BadRequest,
        PasswordError::InvalidHash | PasswordError::Hashing | PasswordError::Task(_) => {
            AuthError::Password(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_email;

    #[test]
    fn normalizes_email() {
        assert_eq!(
            normalize_email(" User@Example.COM ").ok().as_deref(),
            Some("user@example.com")
        );
    }

    #[test]
    fn rejects_invalid_email() {
        assert!(normalize_email("invalid").is_err());
        assert!(normalize_email("@").is_err());
        assert!(normalize_email("user@").is_err());
        assert!(normalize_email("@example.com").is_err());
        assert!(normalize_email("user @example.com").is_err());
        assert!(normalize_email("a@b@example.com").is_err());
    }
}
