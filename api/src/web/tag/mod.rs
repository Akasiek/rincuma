mod create;
mod delete;
mod get;
mod list;
mod request;
mod response;
mod routes;
mod update;

pub(crate) use routes::router;
pub(super) type TagError = super::error::ResourceError<crate::db::Tag>;
