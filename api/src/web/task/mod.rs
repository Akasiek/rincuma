mod handlers;
mod relations;
mod routes;

pub(super) use routes::router;
pub(super) type TaskError = super::error::ResourceError<crate::db::Task>;
