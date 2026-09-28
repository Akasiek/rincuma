use super::{task::Task, user::User};
use jiff::Timestamp;
use toasty::{Deferred, Model};

#[derive(Debug, Model)]
pub(crate) struct Project {
    #[key]
    #[auto]
    pub(crate) id: i64,

    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) color: Option<String>,
    pub(crate) archived_at: Option<Timestamp>,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,

    #[index]
    pub(crate) owner_id: i64,
    #[belongs_to(key = owner_id, references = id)]
    pub(crate) owner: Deferred<User>,

    #[has_many]
    pub(crate) tasks: Deferred<Vec<Task>>,
}

impl Project {
    pub(crate) async fn get_owned(
        executor: &mut dyn toasty::Executor,
        id: i64,
        owner_id: i64,
    ) -> toasty::Result<Option<Self>> {
        Ok(Self::filter_by_id(id)
            .first()
            .exec(executor)
            .await?
            .filter(|project| project.owner_id == owner_id))
    }
}
