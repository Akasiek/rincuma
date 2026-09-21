use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct AppConfig {
    pub(super) data_dir: PathBuf,
    pub(super) database_url: String,
    pub(super) listen_address: SocketAddr,
}

impl AppConfig {
    #[cfg(test)]
    pub(crate) fn for_test(library_dir: PathBuf) -> Self {
        Self {
            data_dir: library_dir.join("data"),
            database_url: "sqlite::memory:".to_owned(),
            listen_address: SocketAddr::from(([127, 0, 0, 1], 0)),
        }
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
}
