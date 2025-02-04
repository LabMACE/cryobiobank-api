#[path = "common/mod.rs"]
mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use common::{build_app_with_db, setup_clean_db};
use cryobiobank_api::sites::models::Site;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn crud_sites() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    // === CREATE ===
    let create_payload = json!({
        "name": "Prabe_S1",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let created_site: Site = serde_json::from_slice(&body_bytes).unwrap();
    let site_id = created_site.id;

    // === READ ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/sites/{}", site_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let fetched_site: Site = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(fetched_site.name, "Prabe_S1");

    // === UPDATE ===
    let update_payload = json!({
        "name": "Prabe_S1_Updated",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1630
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/sites/{}", site_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let updated_site: Site = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(updated_site.name, "Prabe_S1_Updated");

    // === DELETE ===
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/sites/{}", site_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // === VERIFY DELETION ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/sites/{}", site_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
