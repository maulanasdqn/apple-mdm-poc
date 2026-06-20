use std::sync::Arc;

use axum::extract::Extension;
use axum::routing::{get, post, put};
use axum::Router;
use tower_http::trace::TraceLayer;

use super::handlers::{admin, checkin, command, enroll, health, scep};
use super::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/enroll", get(enroll::enroll).post(enroll::enroll))
        .route("/checkin", put(checkin::checkin).post(checkin::checkin))
        .route("/server", put(command::command).post(command::command))
        .route("/scep", get(scep::scep).post(scep::scep))
        .route("/scep/issue", post(scep::issue))
        .route("/admin/devices", get(admin::list_devices))
        .route("/admin/commands/{udid}", post(admin::enqueue_command))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state))
}
