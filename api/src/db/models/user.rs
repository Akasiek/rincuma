#[derive(Debug, toasty::Model)]
pub(crate) struct User {
    #[key]
    #[auto]
    pub(crate) id: i64,

    #[unique]
    pub(crate) email: String,

    pub(crate) password_hash: String,

    #[default(true)]
    pub(crate) is_active: bool,

    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}
