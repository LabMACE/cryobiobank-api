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
async fn crud_dna() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db.clone());

    // === CREATE DNA ===
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
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let dna: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let dna_id = dna.get("id").unwrap().as_str().unwrap().to_string();

    // === READ ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/dna/{}", dna_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(fetched.get("name").unwrap().as_str().unwrap(), "gDNA A");

    // === UPDATE ===
    let update_payload = json!({
        "name": "gDNA A Updated",
        "description": "From Isolate A - Updated",
        "extraction_method": "Genomic DNA"
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/dna/{}", dna_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        updated.get("name").unwrap().as_str().unwrap(),
        "gDNA A Updated"
    );

    // === DELETE ===
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/dna/{}", dna_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // === VERIFY DELETION ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/dna/{}", dna_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
