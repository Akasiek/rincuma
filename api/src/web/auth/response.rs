use serde::Serialize;

use crate::db::User;

#[derive(Serialize)]
pub(in crate::web::auth) struct UserResponse {
    id: i64,
    email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
        }
    }
}
