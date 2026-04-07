use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use crate::sites::db::Site;
use serde_json::json;
use tower::ServiceExt;

use crate::test_utils::{build_app_with_db, setup_clean_db};

#[tokio::test]
#[ignore]
async fn crud_sites() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    // === CREATE ===
    let create_payload = json!({
        "name": "Test Site",
        "latitude_4326": 46.0,
        "longitude_4326": 7.0,
        "elevation_metres": 100
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

    let created_site: Site =
        serde_json::from_slice(&body_bytes).expect("Failed to parse site creation response");
    let site_id = created_site.id;

    // === READ (GET by ID) ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/sites/{}", site_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();

    let fetched_site: Site =
        serde_json::from_slice(&body_bytes).expect("Failed to parse site fetch response");
    assert_eq!(fetched_site.name, "Test Site");

    // === UPDATE ===
    let update_payload = json!({
        "name": "Updated Site",
        "latitude_4326": 46.0,
        "longitude_4326": 7.0,
        "elevation_metres": 100
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

    let updated_site: Site =
        serde_json::from_slice(&body_bytes).expect("Failed to parse site update response");
    assert_eq!(updated_site.name, "Updated Site");

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

#[tokio::test]
#[ignore]
async fn create_site_valid() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

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
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

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
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn create_site_invalid_longitude() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

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
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

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
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let invalid_lat_payload = json!({
        "name": "Invalid_Lat",
        "latitude_4326": 100.0,
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

    let invalid_long_payload = json!({
        "name": "Invalid_Long",
        "latitude_4326": 46.27095,
        "longitude_4326": -200.0,
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

    let valid_payload = json!({
        "name": "Unique_Site",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
#[ignore]
async fn test_sites_invalid_update() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

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

    let update_payload_invalid = json!({
        "name": "Prabe_S3_Invalid",
        "latitude_4326": 100.0,
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

    let update_payload_invalid = json!({
        "name": "Prabe_S3_Invalid",
        "latitude_4326": 40.0,
        "longitude_4326": -200.0,
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
