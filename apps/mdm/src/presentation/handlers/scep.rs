use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Extension, Query};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::infrastructure::cert::scep::{ca_caps, ScepOperation};
use crate::presentation::{error::ApiError, AppState};

use super::content_type;

#[derive(Debug, Deserialize)]
pub struct ScepQuery {
    operation: Option<String>,
}

pub async fn scep(
    Extension(state): Extension<Arc<AppState>>,
    Query(q): Query<ScepQuery>,
) -> Result<Response, ApiError> {
    let op = ScepOperation::from_query(q.operation.as_deref().unwrap_or(""));
    match op {
        ScepOperation::GetCACert => {
            let der = state.cert.ca_cert_der()?;
            Ok(([(header::CONTENT_TYPE, content_type::CA_CERT)], der).into_response())
        }
        ScepOperation::GetCACaps => Ok((
            [(header::CONTENT_TYPE, "text/plain")],
            ca_caps().to_string(),
        )
            .into_response()),
        ScepOperation::PkiOperation => Ok((
            StatusCode::NOT_IMPLEMENTED,
            "SCEP PKIOperation transport (PKCS#7 envelope with SCEP signed \
             attributes) is not implemented. The CA issuance core is available \
             at POST /scep/issue — send a PKCS#10 CSR (PEM or DER).",
        )
            .into_response()),
        ScepOperation::Unknown => Err(ApiError::BadRequest("unknown SCEP operation".to_string())),
    }
}

pub async fn issue(
    Extension(state): Extension<Arc<AppState>>,
    body: Bytes,
) -> Result<Response, ApiError> {
    if body.is_empty() {
        return Err(ApiError::BadRequest("empty CSR body".into()));
    }
    let cert_der = state.cert.issue_identity(&body)?;
    Ok(([(header::CONTENT_TYPE, "application/pkix-cert")], cert_der).into_response())
}
