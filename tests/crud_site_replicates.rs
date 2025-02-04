#[path = "common/mod.rs"]
mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn create_site_replicate_valid() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create a parent site.
    let create_site_payload = json!({
        "name": "Prabe_S1",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let site_request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let site_response = app.clone().oneshot(site_request).await.unwrap();
    assert_eq!(site_response.status(), StatusCode::CREATED);
    let site_body = to_bytes(site_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_body).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    // Valid site replicate.
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
}

#[tokio::test]
async fn create_site_replicate_invalid_date() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create a parent site.
    let create_site_payload = json!({
        "name": "Prabe_S2",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let site_request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let site_response = app.clone().oneshot(site_request).await.unwrap();
    assert_eq!(site_response.status(), StatusCode::CREATED);
    let site_body = to_bytes(site_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_body).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    // Try to create a site replicate with an invalid sampling_date.
    let create_payload = json!({
        "site_id": site_id,
        "name": "P2S1-T_Invalid",
        "sample_type": "Snow",
        "sampling_date": "2023-02-30"  // February 30 is invalid.
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_site_replicates_invalid_data() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create a valid parent Site first.
    let create_site_payload = json!({
        "name": "Parent_Site",
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
    let site_id = site.get("id").unwrap().as_str().unwrap();

    // Test an invalid sampling_date (e.g. "2023-02-30" is not a valid date)
    let invalid_date_payload = json!({
        "site_id": site_id,
        "name": "Replicate_Invalid_Date",
        "sample_type": "Snow",
        "sampling_date": "2023-02-30"  // invalid date
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_date_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Test missing required field (e.g. missing "sample_type")
    let missing_field_payload = json!({
        "site_id": site_id,
        "name": "Replicate_Missing_SampleType",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(missing_field_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
