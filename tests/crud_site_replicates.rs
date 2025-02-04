#[path = "common/mod.rs"]
mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use common::{build_app_with_db, setup_clean_db};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn crud_site_replicates() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    // Create a parent Site.
    let create_site_payload = json!({
        "name": "Prabe_S1",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap().to_string();

    // === CREATE site_replicate ===
    let create_payload = json!({
        "site_id": site_id,
        "name": "P2S1-T",
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap().to_string();

    // === READ ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/site_replicates/{}", replicate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(fetched.get("name").unwrap().as_str().unwrap(), "P2S1-T");

    // === UPDATE ===
    let update_payload = json!({
        "name": "P2S1-T-Updated",
        "site_id": site_id,
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/site_replicates/{}", replicate_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        updated.get("name").unwrap().as_str().unwrap(),
        "P2S1-T-Updated"
    );

    // === DELETE ===
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/site_replicates/{}", replicate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // === VERIFY DELETION ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/site_replicates/{}", replicate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
