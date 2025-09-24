#[path = "common/mod.rs"]
mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use cryobiobank_api::sites::models::Site;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
#[ignore]
async fn create_site_valid() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    let create_payload = json!({
        "name": "Prabe_S1",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
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
    assert_eq!(created_site.name, "Prabe_S1");
    assert_eq!(created_site.latitude_4326, 46.27095);
    assert_eq!(created_site.longitude_4326, 7.3349);
    assert_eq!(created_site.elevation_metres, 1629.0);
}

#[tokio::test]
#[ignore]
async fn create_site_invalid_latitude() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Latitude > 90 is invalid.
    let create_payload = json!({
        "name": "Invalid_Latitude",
        "latitude_4326": 100.0,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    // Expect a 400 BAD REQUEST response for invalid latitude.
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn create_site_invalid_longitude() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Longitude < -180 or > 180 is invalid. Here we use -190.
    let create_payload = json!({
        "name": "Invalid_Longitude",
        "latitude_4326": 46.27095,
        "longitude_4326": -190.0,
        "elevation_metres": 1629.0
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn update_site_varied_data() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create a valid site.
    let create_payload = json!({
        "name": "Prabe_S2",
        "latitude_4326": 40.0,
        "longitude_4326": 10.0,
        "elevation_metres": 500.0
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

    // Update with valid varied data.
    let update_payload = json!({
        "name": "Prabe_S2_Updated",
        "latitude_4326": 39.5,
        "longitude_4326": 9.8,
        "elevation_metres": 505.5
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
    assert_eq!(updated_site.name, "Prabe_S2_Updated");
    assert_eq!(updated_site.latitude_4326, 39.5);
    assert_eq!(updated_site.longitude_4326, 9.8);
    assert_eq!(updated_site.elevation_metres, 505.5);

    // Now test update with negative elevation
    let update_payload_invalid = json!({
        "name": "Prabe_S2_Invalid",
        "latitude_4326": 39.5,
        "longitude_4326": 9.8,
        "elevation_metres": -10.0
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/sites/{}", site_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload_invalid.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_sites_invalid_values() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Test invalid latitude (valid range is assumed to be -90 to 90)
    let invalid_lat_payload = json!({
        "name": "Invalid_Lat",
        "latitude_4326": 100.0, // invalid
        "longitude_4326": 7.3349,
        "elevation_metres": 1629
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_lat_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test invalid longitude (valid range is assumed to be -180 to 180)
    let invalid_long_payload = json!({
        "name": "Invalid_Long",
        "latitude_4326": 46.27095,
        "longitude_4326": -200.0, // invalid
        "elevation_metres": 1629
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_long_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test duplicate name (unique constraint)
    let valid_payload = json!({
        "name": "Unique_Site",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629
    });
    // First creation should succeed
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second creation with the same name should fail
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// Test update with incorrect lat/long values
#[tokio::test]
#[ignore]
async fn test_sites_invalid_update() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    // Create a valid site.
    let create_payload = json!({
        "name": "Prabe_S3",
        "latitude_4326": 40.0,
        "longitude_4326": 10.0,
        "elevation_metres": 500.0
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

    // Update with invalid latitude
    let update_payload_invalid = json!({
        "name": "Prabe_S3_Invalid",
        "latitude_4326": 100.0, // invalid
        "longitude_4326": 10.0,
        "elevation_metres": 500.0
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/sites/{}", site_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload_invalid.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Update with invalid longitude
    let update_payload_invalid = json!({
        "name": "Prabe_S3_Invalid",
        "latitude_4326": 40.0,
        "longitude_4326": -200.0, // invalid
        "elevation_metres": 500.0
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/sites/{}", site_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload_invalid.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
