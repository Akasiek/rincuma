use super::user::User;
use toasty::{Deferred, Model};

#[derive(Model)]
pub(crate) struct NavigationPreferences {
    #[key]
    pub(crate) owner_id: i64,
    #[belongs_to(key = owner_id, references = id)]
    pub(crate) owner: Deferred<User>,
    pub(crate) projects_open: Option<bool>,
    pub(crate) tags_open: Option<bool>,
}
