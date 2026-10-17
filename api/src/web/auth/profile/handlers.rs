use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

use super::service::{save_password, save_profile};
use crate::{
    app_state::AppState,
    auth::password::{hash_password, verify_password},
    web::auth::{
        error::AuthError,
        extractor::CurrentUser,
        response::UserResponse,
        session_cookie::session_token,
        validation::{map_password_error, normalize_email},
    },
};

#[derive(Deserialize)]
pub(in crate::web::auth) struct UpdateProfile {
    pub(super) email: String,
}

#[derive(Deserialize)]
pub(in crate::web::auth) struct ChangePassword {
    pub(super) current_password: String,
    pub(super) new_password: String,
    pub(super) password_confirmation: String,
}

pub(in crate::web::auth) async fn update_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    headers: HeaderMap,
    Json(request): Json<UpdateProfile>,
) -> Result<Json<UserResponse>, AuthError> {
    let email = normalize_email(&request.email)?;
    let token =
        session_token(&headers, state.config().cookie_secure()).ok_or(AuthError::Unauthorized)?;
    let updated = save_profile(&state, &user, &token, &email).await?;
    Ok(Json(updated.into()))
}

pub(in crate::web::auth) async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    headers: HeaderMap,
    Json(request): Json<ChangePassword>,
) -> Result<StatusCode, AuthError> {
    if request.new_password != request.password_confirmation {
        return Err(AuthError::BadRequest);
    }
    let token =
        session_token(&headers, state.config().cookie_secure()).ok_or(AuthError::Unauthorized)?;
    let valid = verify_password(request.current_password, user.password_hash.clone())
        .await
        .map_err(map_password_error)?;
    if !valid {
        return Err(AuthError::InvalidPassword);
    }
    let password_hash = hash_password(request.new_password)
        .await
        .map_err(map_password_error)?;
    save_password(&state, &user, &token, &password_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}
