use std::time::SystemTime;

use crate::web::auth::error::AuthError;

pub(super) fn current_timestamp() -> Result<jiff::Timestamp, AuthError> {
    jiff::Timestamp::try_from(SystemTime::now()).map_err(|_| AuthError::Internal)
}
