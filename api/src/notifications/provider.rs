use super::ntfy::Ntfy;
use crate::db::{NotificationProviderKind, NotificationSettings};
use std::{future::Future, pin::Pin, time::Duration};

pub(crate) struct Notification {
    pub(crate) title: String,
    pub(crate) message: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum NotificationError {
    #[error("notification transport failed")]
    Transport,
    #[error("notification provider returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("invalid notification configuration")]
    Configuration,
}

pub(crate) trait NotificationProvider: Send + Sync {
    fn send<'a>(
        &'a self,
        settings: &'a NotificationSettings,
        notification: &'a Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), NotificationError>> + Send + 'a>>;
}

pub(crate) struct Providers {
    ntfy: Ntfy,
}

impl Providers {
    pub(crate) fn new() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            ntfy: Ntfy::new(client),
        })
    }

    pub(crate) fn get(&self, kind: NotificationProviderKind) -> &dyn NotificationProvider {
        match kind {
            NotificationProviderKind::Ntfy => &self.ntfy,
        }
    }
}
