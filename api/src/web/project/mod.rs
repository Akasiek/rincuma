mod create;
mod delete;
mod get;
mod list;
mod request;
mod response;
mod routes;
mod update;

pub(crate) use routes::router;
pub(super) type ProjectError = super::error::ResourceError<crate::db::Project>;
