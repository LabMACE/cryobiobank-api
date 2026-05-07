use super::models::HealthCheck;
use crate::common::models::UIConfiguration;

use axum::{extract::State, http::StatusCode, Json};
use sea_orm::DatabaseConnection;

#[utoipa::path(
    get,
    path = "/api/healthz",
    responses(
        (
            status = OK,
            description = "Kubernetes health check",
            body = str,
            content_type = "text/plain"
        )
    )
)]
pub async fn healthz(State(db): State<DatabaseConnection>) -> (StatusCode, Json<HealthCheck>) {
    if let Err(_) = db.ping().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HealthCheck {
                status: "error".to_string(),
            }),
        )
    };

    (
        StatusCode::OK,
        Json(HealthCheck {
            status: "ok".to_string(),
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/config",
    responses(
        (
            status = OK,
            description = "Web UI configuration",
            body = str,
            content_type = "text/plain"
        )
    )
)]
pub async fn get_ui_config() -> Json<UIConfiguration> {
    Json(UIConfiguration::new())
}
