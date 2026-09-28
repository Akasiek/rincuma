mod create;
mod get;
mod list;
mod request;
mod response;
mod routes;

pub(crate) use routes::router;
pub(super) type TagError = super::error::ResourceError<crate::db::Tag>;
