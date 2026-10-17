#[derive(Debug, toasty::Model)]
pub(crate) struct Session {
    #[key]
    #[auto]
    pub(crate) id: i64,

    #[unique]
    pub(crate) token_hash: String,

    #[index]
    pub(crate) user_id: i64,

    pub(crate) expires_at: jiff::Timestamp,
    pub(crate) created_at: jiff::Timestamp,
}

impl Session {
    pub(crate) async fn is_valid_for_user(
        executor: &mut dyn toasty::Executor,
        token_hash: &str,
        user_id: i64,
        now: jiff::Timestamp,
    ) -> toasty::Result<bool> {
        Ok(Self::filter_by_token_hash(token_hash.to_owned())
            .filter(Self::fields().user_id().eq(user_id))
            .filter(Self::fields().expires_at().gt(now))
            .first()
            .exec(executor)
            .await?
            .is_some())
    }

    pub(crate) async fn delete_other_sessions(
        executor: &mut dyn toasty::Executor,
        user_id: i64,
        current_token_hash: &str,
    ) -> toasty::Result<()> {
        Self::filter_by_user_id(user_id)
            .filter(
                Self::fields()
                    .token_hash()
                    .ne(current_token_hash.to_owned()),
            )
            .delete()
            .exec(executor)
            .await?;
        Ok(())
    }
}
