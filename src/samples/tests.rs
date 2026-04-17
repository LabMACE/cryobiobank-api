use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::test_utils::{build_app_with_db, setup_clean_db};

#[tokio::test]
#[ignore]
async fn crud_samples() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

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
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap().to_string();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "P2S1-T",
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
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap().to_string();

    let create_payload = json!({
        "name": "P2S1-T",
        "site_replicate_id": replicate_id,
        "storage_location": "Samples: A1",
        "description": "Initial sample"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/samples")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let sample: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let sample_id = sample.get("id").unwrap().as_str().unwrap().to_string();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/samples/{}", sample_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(fetched.get("name").unwrap().as_str().unwrap(), "P2S1-T");

    let update_payload = json!({
        "name": "P2S1-T-Updated",
        "site_replicate_id": replicate_id,
        "storage_location": "Samples: A1",
        "description": "Updated sample"
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/samples/{}", sample_id))
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

    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/samples/{}", sample_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/samples/{}", sample_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_samples_invalid_and_duplicate() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let create_site_payload = json!({
        "name": "Sample_Parent_Site",
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
        "name": "Sample_Replicate",
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

    let invalid_uuid_payload = json!({
        "name": "Sample_Invalid_UUID",
        "site_replicate_id": "invalid-uuid",
        "storage_location": "Samples: Test",
        "description": "Test sample"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/samples")
        .header("Content-Type", "application/json")
        .body(Body::from(invalid_uuid_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let valid_payload = json!({
        "name": "Unique_Sample",
        "site_replicate_id": replicate_id,
        "storage_location": "Samples: Test",
        "description": "Test sample"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/samples")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let request = Request::builder()
        .method("POST")
        .uri("/api/samples")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
