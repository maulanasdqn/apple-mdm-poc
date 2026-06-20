use axum::body::Bytes;
use axum::extract::FromRequest;
use axum::http::Request;
use axum::response::{IntoResponse, Response};

use super::error::ApiError;

pub struct PlistBody(pub Vec<u8>);

impl<S> FromRequest<S> for PlistBody
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ApiError::BadRequest(format!("read body: {e}")).into_response())?;
        Ok(PlistBody(bytes.to_vec()))
    }
}
