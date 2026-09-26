use crate::{
    auth::token::{SessionToken, TokenGenerationError},
    db::Session,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum SessionError {
    #[error("failed to generate a session token")]
    Token(#[from] TokenGenerationError),
    #[error("failed to store a session")]
    Database(#[from] toasty::Error),
}

pub(crate) const SESSION_DURATION_SECONDS: i64 = 60 * 60 * 24 * 30;

pub(crate) async fn create_session(
    executor: &mut dyn toasty::Executor,
    user_id: i64,
    now: i64,
) -> Result<SessionToken, SessionError> {
    let token = SessionToken::generate()?;

    toasty::create!(Session {
        token_hash: token.digest(),
        user_id,
        expires_at: now + SESSION_DURATION_SECONDS,
        created_at: now,
    })
    .exec(executor)
    .await
    .map_err(SessionError::from)?;

    Ok(token)
}
