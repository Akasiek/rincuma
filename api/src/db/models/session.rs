#[derive(Debug, toasty::Model)]
pub(crate) struct Session {
    #[key]
    #[auto]
    pub(crate) id: i64,

    #[unique]
    pub(crate) token_hash: String,

    #[index]
    pub(crate) user_id: i64,

    pub(crate) expires_at: i64,
    pub(crate) created_at: i64,
}
