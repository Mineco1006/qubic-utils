use axum::response::{IntoResponse, Response};
use http::status::StatusCode;

mod serializeable_types;

pub use serializeable_types::*;

pub struct QubicRpcError(anyhow::Error);

impl IntoResponse for QubicRpcError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

// Enables using `?` on functions that return `Result<_, anyhow::Error>`
impl<E> From<E> for QubicRpcError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}