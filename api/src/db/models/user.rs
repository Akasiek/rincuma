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

    pub(crate) created_at: jiff::Timestamp,
    pub(crate) updated_at: jiff::Timestamp,
}

impl User {
    pub(crate) async fn is_email_taken(
        executor: &mut dyn toasty::Executor,
        email: &str,
        user_id: i64,
    ) -> toasty::Result<bool> {
        Ok(Self::filter_by_email(email.to_owned())
            .filter(Self::fields().id().ne(user_id))
            .first()
            .exec(executor)
            .await?
            .is_some())
    }

    /// Updates only the profile version that the caller originally read.
    pub(crate) async fn change_email(
        &self,
        executor: &mut dyn toasty::Executor,
        email: &str,
        now: jiff::Timestamp,
    ) -> toasty::Result<()> {
        toasty::update!(Self::filter_by_id(self.id)
            .filter(Self::fields().updated_at().eq(self.updated_at)) {
            email: email.to_owned(),
            updated_at: now,
        })
        .exec(executor)
        .await
    }

    /// Prevents an in-flight request from overwriting a newer password change.
    pub(crate) async fn change_password(
        &self,
        executor: &mut dyn toasty::Executor,
        password_hash: &str,
        now: jiff::Timestamp,
    ) -> toasty::Result<()> {
        toasty::update!(Self::filter_by_id(self.id)
            .filter(Self::fields().updated_at().eq(self.updated_at)) {
            password_hash: password_hash.to_owned(),
            updated_at: now,
        })
        .exec(executor)
        .await
    }
}
