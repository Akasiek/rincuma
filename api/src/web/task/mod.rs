mod completion;
mod deletion;
mod handlers;
mod history;
mod locking;
mod recurrence;
mod relations;
mod reopening;
mod request;
mod response;
mod routes;

pub(super) use routes::router;
pub(super) type TaskError = super::error::ResourceError<crate::db::Task>;

#[cfg(test)]
#[path = "tests/recurring_support.rs"]
mod recurring_support;

#[cfg(test)]
#[path = "tests/recurring_reopening_postgres.rs"]
mod recurring_reopening_postgres;
