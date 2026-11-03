use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::config::AdminCredentials;

#[derive(Debug)]
pub(crate) struct AppConfig {
    pub(super) data_dir: PathBuf,
    pub(super) database_url: String,
    pub(super) listen_address: SocketAddr,
    pub(super) cookie_secure: bool,
    pub(super) registration_enabled: bool,
    pub(super) admin: Option<AdminCredentials>,
}

impl AppConfig {
    #[cfg(test)]
    pub(crate) fn for_test(library_dir: PathBuf) -> Self {
        Self {
            data_dir: library_dir.join("data"),
            database_url: "sqlite::memory:".to_owned(),
            listen_address: SocketAddr::from(([127, 0, 0, 1], 0)),
            cookie_secure: false,
            registration_enabled: false,
            admin: None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_registration_enabled(mut self) -> Self {
        self.registration_enabled = true;
        self
    }

    pub(crate) fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    pub(crate) const fn listen_address(&self) -> SocketAddr {
        self.listen_address
    }

    pub(crate) const fn cookie_secure(&self) -> bool {
        self.cookie_secure
    }

    pub(crate) const fn registration_enabled(&self) -> bool {
        self.registration_enabled
    }

    /// Moves the admin credentials out so the password is not kept for the process lifetime.
    pub(crate) const fn take_admin(&mut self) -> Option<AdminCredentials> {
        self.admin.take()
    }
}
