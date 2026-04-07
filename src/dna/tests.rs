use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::test_utils::{build_app_with_db, setup_clean_db};

#[tokio::test]
#[ignore]
async fn create_dna_valid() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let create_site_payload = json!({
        "name": "DNA_Parent_Site",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let site_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_body).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "DNA_Replicate",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let replicate_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&replicate_body).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    let create_payload = json!({
        "name": "gDNA A",
        "description": "From Isolate A",
        "extraction_method": "Genomic DNA",
        "site_replicate_id": replicate_id
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/dna")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore]
async fn test_dna_duplicate_name() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    let create_site_payload = json!({
        "name": "DNA_Dup_Site",
        "latitude_4326": 46.27095,
        "longitude_4326": 7.3349,
        "elevation_metres": 1629.0
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/sites")
        .header("Content-Type", "application/json")
        .body(Body::from(create_site_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let site_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&site_body).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap();

    let create_replicate_payload = json!({
        "site_id": site_id,
        "name": "DNA_Dup_Replicate",
        "sampling_date": "2023-02-18"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/site_replicates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_replicate_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let replicate_body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let replicate: serde_json::Value = serde_json::from_slice(&replicate_body).unwrap();
    let replicate_id = replicate.get("id").unwrap().as_str().unwrap();

    let create_payload = json!({
        "name": "gDNA Duplicate",
        "description": "Test DNA",
        "extraction_method": "Genomic DNA",
        "site_replicate_id": replicate_id
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/dna")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let request = Request::builder()
        .method("POST")
        .uri("/api/dna")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
