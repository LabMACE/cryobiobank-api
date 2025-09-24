#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
#[ignore]
async fn create_dna_valid() {
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    let create_payload = json!({
        "name": "gDNA A",
        "description": "From Isolate A",
        "extraction_method": "Genomic DNA"
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
    let db = common::setup_clean_db().await;
    let app = common::build_app_with_db(db.clone());

    let create_payload = json!({
        "name": "gDNA Duplicate",
        "description": "Test DNA",
        "extraction_method": "Genomic DNA"
    });
    // First creation should succeed.
    let request = Request::builder()
        .method("POST")
        .uri("/api/dna")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second creation with the same name should fail.
    let request = Request::builder()
        .method("POST")
        .uri("/api/dna")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
