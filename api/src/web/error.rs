use std::marker::PhantomData;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

use crate::db::{Project, Tag, Task};

pub(crate) trait ResourceName: std::fmt::Debug {
    const NAME: &'static str;
}

impl ResourceName for Project {
    const NAME: &'static str = "project";
}

impl ResourceName for Task {
    const NAME: &'static str = "task";
}

impl ResourceName for Tag {
    const NAME: &'static str = "tag";
}

#[derive(Debug, Error)]
pub(crate) enum ResourceError<R: ResourceName> {
    #[error("invalid {} request", R::NAME)]
    BadRequest(PhantomData<R>),
    #[error("related resource not found")]
    RelatedResourceNotFound(PhantomData<R>),
    #[error("{} database operation failed", R::NAME)]
    Database(#[source] toasty::Error, PhantomData<R>),
}

impl<R: ResourceName> ResourceError<R> {
    pub(crate) fn bad_request() -> Self {
        Self::BadRequest(PhantomData)
    }

    pub(crate) fn related_resource_not_found() -> Self {
        Self::RelatedResourceNotFound(PhantomData)
    }
}

impl<R: ResourceName> From<toasty::Error> for ResourceError<R> {
    fn from(error: toasty::Error) -> Self {
        Self::Database(error, PhantomData)
    }
}

impl<R: ResourceName> IntoResponse for ResourceError<R> {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::RelatedResourceNotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(..) => {
                error!(error = ?self, "{} request failed", R::NAME);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use super::ResourceError;
    use crate::db::{Project, Tag, Task};

    #[test]
    fn errors_keep_resource_specific_messages_and_statuses() {
        let project = ResourceError::<Project>::bad_request();
        assert_eq!(project.to_string(), "invalid project request");
        assert_eq!(project.into_response().status(), StatusCode::BAD_REQUEST);

        let task = ResourceError::<Task>::bad_request();
        assert_eq!(task.to_string(), "invalid task request");
        assert_eq!(task.into_response().status(), StatusCode::BAD_REQUEST);

        let tag = ResourceError::<Tag>::bad_request();
        assert_eq!(tag.to_string(), "invalid tag request");
        assert_eq!(tag.into_response().status(), StatusCode::BAD_REQUEST);

        let missing = ResourceError::<Task>::related_resource_not_found();
        assert_eq!(missing.to_string(), "related resource not found");
        assert_eq!(missing.into_response().status(), StatusCode::NOT_FOUND);
    }
}
