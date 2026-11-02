use super::{
    request::{SaveSection, Section},
    response::PreferencesResponse,
};
use crate::{app_state::AppState, db::NavigationPreferences, web::auth::CurrentUser};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

pub(super) async fn get_preferences(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<PreferencesResponse>, StatusCode> {
    let preferences = NavigationPreferences::filter_by_owner_id(user.id)
        .first()
        .exec(&mut state.db().clone())
        .await
        .map_err(|error| database_error(&error))?;

    Ok(Json(preferences.map(Into::into).unwrap_or_default()))
}

pub(super) async fn save_section(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(section): Path<Section>,
    Json(request): Json<SaveSection>,
) -> Result<StatusCode, StatusCode> {
    let update = NavigationPreferences::upsert_by_owner_id(user.id);
    let update = match section {
        Section::Projects => update.projects_open(Some(request.open)),
        Section::Tags => update.tags_open(Some(request.open)),
    };
    update
        .exec(&mut state.db().clone())
        .await
        .map_err(|error| database_error(&error))?;

    Ok(StatusCode::NO_CONTENT)
}

fn database_error(error: &toasty::Error) -> StatusCode {
    tracing::error!(error = %error, "Navigation preferences database operation failed");
    StatusCode::INTERNAL_SERVER_ERROR
}
