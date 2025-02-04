#[path = "common/mod.rs"]
mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn create_isolate_valid() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create parent site.
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

    // Create parent site replicate.
    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "P2S1-T",
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let replicate_request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let replicate_response = app.clone().oneshot(replicate_request).await.unwrap();
    assert_eq!(replicate_response.status(), StatusCode::CREATED);
    let replicate_body = to_bytes(replicate_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&replicate_body).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    // Create a valid isolate.
    let create_payload = json!({
        "name": "Isolate A",
        "site_replicate_id": replicate_id,
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": 20.5,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: A1",
        "dna_id": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_isolates_invalid_data() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create valid parent Site and Site Replicate.
    let create_site_payload = json!({
        "name": "Isolate_Parent_Site",
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
    let site_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_bytes).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "Isolate_Replicate",
        "sample_type": "Snow",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let rep_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&rep_bytes).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    // Test invalid temperature_of_isolation (string instead of a number)
    let invalid_temp_payload = json!({
        "name": "Isolate_Invalid_Temp",
        "site_replicate_id": replicate_id,
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": "hot",  // invalid
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: Test",
        "dna_id": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_temp_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Test invalid site_replicate_id (malformed UUID)
    let invalid_uuid_payload = json!({
        "name": "Isolate_Invalid_UUID",
        "site_replicate_id": "not-a-uuid",
        "taxonomy": "Pseudomonas",
        "photo": "",
        "temperature_of_isolation": 20.5,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: Test",
        "dna_id": null
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_uuid_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
