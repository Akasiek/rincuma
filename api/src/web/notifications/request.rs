use crate::db::NotificationProviderKind;
use axum::http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct SaveSettings {
    pub(super) provider: NotificationProviderKind,
    pub(super) enabled: bool,
    pub(super) url: String,
    /// Omitted/null preserves the credential; an empty string clears it.
    pub(super) authorization: Option<String>,
}

pub(super) fn validate_settings(request: &SaveSettings) -> Result<(), StatusCode> {
    let url = reqwest::Url::parse(&request.url).map_err(|_| StatusCode::BAD_REQUEST)?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path().trim_matches('/').is_empty()
        || request.url.len() > 2048
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(value) = &request.authorization
        && (value.len() > 4096 || reqwest::header::HeaderValue::from_str(value).is_err())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}
