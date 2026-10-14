use super::user::User;
use jiff::Timestamp;
use toasty::stmt::{List, Query};
use toasty::{Deferred, Model};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NotificationProviderKind {
    Ntfy,
}

impl NotificationProviderKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Ntfy => "ntfy",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        match value {
            "ntfy" => Some(Self::Ntfy),
            _ => None,
        }
    }
}

#[derive(Model)]
pub(crate) struct NotificationSettings {
    #[key]
    pub(crate) owner_id: i64,
    #[belongs_to(key = owner_id, references = id)]
    pub(crate) owner: Deferred<User>,
    pub(crate) provider: String,
    pub(crate) enabled: bool,
    pub(crate) url: String,
    pub(crate) authorization: Option<String>,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,
}

impl NotificationSettings {
    /// Returns a batch of enabled notification settings ordered by owner ID.
    ///
    /// The background worker scans settings across users, then processes each
    /// user's tasks with their own notification destination and credentials.
    /// Pass `0` for the first batch. For subsequent batches, pass the last
    /// returned owner ID: the strict `owner_id > last_processed_owner_id` filter
    /// excludes previously processed settings. An empty batch ends the scan.
    pub(crate) async fn find_enabled_batch_after_owner_id(
        executor: &mut dyn toasty::Executor,
        last_processed_owner_id: i64,
        limit: usize,
    ) -> toasty::Result<Vec<Self>> {
        Query::<List<Self>>::all()
            .filter(Self::fields().enabled().eq(true))
            .filter(Self::fields().owner_id().gt(last_processed_owner_id))
            .order_by(Self::fields().owner_id().asc())
            .limit(limit)
            .exec(executor)
            .await
    }
}

impl std::fmt::Debug for NotificationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotificationSettings")
            .field("owner_id", &self.owner_id)
            .field("provider", &self.provider)
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}
