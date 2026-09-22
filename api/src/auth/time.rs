use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::error::AuthError;

pub(super) fn current_unix_timestamp_seconds() -> Result<i64, AuthError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthError::Internal)?
        .as_secs();

    i64::try_from(seconds).map_err(|_| AuthError::Internal)
}
