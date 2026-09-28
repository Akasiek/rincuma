mod create;
mod delete;
mod get;
mod list;
mod relations;
mod request;
mod response;
mod routes;
mod update;

pub(super) use routes::router;
pub(super) type TaskError = super::error::ResourceError<crate::db::Task>;
