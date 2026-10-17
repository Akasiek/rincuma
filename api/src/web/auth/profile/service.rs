use crate::{
    app_state::AppState,
    auth::token::SessionToken,
    db::{Session, User},
    web::auth::{error::AuthError, time::current_timestamp},
};

pub(super) async fn save_profile(
    state: &AppState,
    user: &User,
    token: &SessionToken,
    email: &str,
) -> Result<User, AuthError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let (token_hash, now) = validate_session(&mut transaction, user.id, token).await?;

    if User::is_email_taken(&mut transaction, email, user.id).await? {
        return Err(AuthError::Conflict);
    }

    if let Err(error) = user.change_email(&mut transaction, email, now).await {
        transaction.rollback().await?;
        if User::is_email_taken(&mut db, email, user.id).await? {
            return Err(AuthError::Conflict);
        }
        return Err(error.into());
    }

    let updated = finalize_change(&mut transaction, user.id, &token_hash, now).await?;
    transaction.commit().await?;
    Ok(updated)
}

pub(super) async fn save_password(
    state: &AppState,
    user: &User,
    token: &SessionToken,
    password_hash: &str,
) -> Result<(), AuthError> {
    let mut db = state.db().clone();
    let mut transaction = db.transaction().await?;
    let (token_hash, now) = validate_session(&mut transaction, user.id, token).await?;

    user.change_password(&mut transaction, password_hash, now)
        .await?;
    finalize_change(&mut transaction, user.id, &token_hash, now).await?;
    transaction.commit().await?;
    Ok(())
}

async fn validate_session(
    executor: &mut dyn toasty::Executor,
    user_id: i64,
    token: &SessionToken,
) -> Result<(String, jiff::Timestamp), AuthError> {
    let token_hash = token.digest();
    let timestamp = current_timestamp()?;

    // Match PostgreSQL precision so the saved timestamp can identify this update.
    let now = jiff::Timestamp::from_microsecond(timestamp.as_microsecond())
        .map_err(|_| AuthError::Internal)?;

    if !Session::is_valid_for_user(executor, &token_hash, user_id, now).await? {
        return Err(AuthError::Unauthorized);
    }
    Ok((token_hash, now))
}

async fn finalize_change(
    executor: &mut dyn toasty::Executor,
    user_id: i64,
    token_hash: &str,
    now: jiff::Timestamp,
) -> Result<User, AuthError> {
    let updated = User::get_by_id(executor, &user_id).await?;
    if updated.updated_at != now {
        return Err(AuthError::StaleProfile);
    }

    Session::delete_other_sessions(executor, user_id, token_hash).await?;
    Ok(updated)
}
