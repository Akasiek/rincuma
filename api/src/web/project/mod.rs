mod handlers;
mod create;
mod routes;

pub(crate) use routes::router;
pub(super) type ProjectError = super::error::ResourceError<crate::db::Project>;
