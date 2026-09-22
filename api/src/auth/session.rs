use crate::{
    auth::{error::AuthError, token::SessionToken},
    db::Session,
};

pub(super) const SESSION_DURATION_SECONDS: i64 = 60 * 60 * 24 * 30;

pub(super) async fn create_session(
    executor: &mut dyn toasty::Executor,
    user_id: i64,
    now: i64,
) -> Result<SessionToken, AuthError> {
    let token = SessionToken::generate().map_err(|_| AuthError::Internal)?;

    toasty::create!(Session {
        token_hash: token.digest(),
        user_id,
        expires_at: now + SESSION_DURATION_SECONDS,
        created_at: now,
    })
    .exec(executor)
    .await
    .map_err(AuthError::from)?;

    Ok(token)
}
