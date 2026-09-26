use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    app_state::AppState,
    db::{Session, User},
    web::auth::{
        error::AuthError, session_cookie::session_token, time::current_unix_timestamp_seconds,
    },
};

pub(crate) struct CurrentUser(pub(crate) User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = session_token(&parts.headers, state.config().cookie_secure())
            .ok_or(AuthError::Unauthorized)?;
        let now = current_unix_timestamp_seconds()?;
        let mut db = state.db().clone();

        let session = Session::filter_by_token_hash(token.digest())
            .first()
            .exec(&mut db)
            .await
            .map_err(AuthError::from)?
            .filter(|session| session.expires_at > now)
            .ok_or(AuthError::Unauthorized)?;

        let user = User::get_by_id(&mut db, &session.user_id)
            .await
            .map_err(AuthError::from)?;

        if !user.is_active {
            return Err(AuthError::Unauthorized);
        }

        Ok(Self(user))
    }
}
