mod create;
mod list;
mod response;
mod routes;

pub(crate) use routes::router;
pub(super) type TagError = super::error::ResourceError<crate::db::Tag>;
