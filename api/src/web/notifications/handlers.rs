use super::{
    request::{SaveSettings, validate_settings},
    response::SettingsResponse,
};
use crate::{app_state::AppState, db::NotificationSettings, web::auth::CurrentUser};
use axum::{Json, extract::State, http::StatusCode};
use jiff::Timestamp;

pub(super) async fn get_settings(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Option<SettingsResponse>>, StatusCode> {
    let settings = NotificationSettings::filter_by_owner_id(user.id)
        .first()
        .exec(&mut state.db().clone())
        .await
        .map_err(|error| database_error(&error))?;

    Ok(Json(settings.map(Into::into)))
}

pub(super) async fn save_settings(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(request): Json<SaveSettings>,
) -> Result<Json<SettingsResponse>, StatusCode> {
    validate_settings(&request)?;
    let mut db = state.db().clone();
    let mut transaction = db
        .transaction()
        .await
        .map_err(|error| database_error(&error))?;
    let existing = NotificationSettings::filter_by_owner_id(user.id)
        .first()
        .exec(&mut transaction)
        .await
        .map_err(|error| database_error(&error))?;

    let authorization = match request.authorization {
        Some(value) => (!value.is_empty()).then_some(value),
        None => existing
            .as_ref()
            .and_then(|value| value.authorization.clone()),
    };

    let now = Timestamp::now();
    let created_at = existing.as_ref().map_or(now, |value| value.created_at);

    let settings = NotificationSettings::upsert_by_owner_id(user.id)
        .provider(request.provider.as_str().to_owned())
        .enabled(request.enabled)
        .url(request.url)
        .authorization(authorization)
        .created_at(created_at)
        .updated_at(now)
        .exec(&mut transaction)
        .await
        .map_err(|error| database_error(&error))?;

    transaction
        .commit()
        .await
        .map_err(|error| database_error(&error))?;
    Ok(Json(settings.into()))
}

pub(super) async fn delete_settings(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<StatusCode, StatusCode> {
    NotificationSettings::filter_by_owner_id(user.id)
        .delete()
        .exec(&mut state.db().clone())
        .await
        .map_err(|error| database_error(&error))?;

    Ok(StatusCode::NO_CONTENT)
}

fn database_error(error: &toasty::Error) -> StatusCode {
    tracing::error!(error = %error, "Notification settings database operation failed");
    StatusCode::INTERNAL_SERVER_ERROR
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
