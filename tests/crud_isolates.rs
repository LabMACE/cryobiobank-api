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
async fn crud_isolates() {
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
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let site: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let site_id = site.get("id").unwrap().as_str().unwrap().to_string();

    // Create a parent Site Replicate.
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

    // Create a parent DNA record.
    let create_dna_payload = json!({
        "name": "gDNA A",
        "description": "From Isolate A",
        "extraction_method": "Genomic DNA"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/dna")
        .header("Content-Type", "application/json")
        .body(Body::from(create_dna_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let dna: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let dna_id = dna.get("id").unwrap().as_str().unwrap().to_string();

    // === CREATE isolate ===
    let create_payload = json!({
        "name": "A",
        "taxonomy": "Pseudomonas",
        "photo": "",
        "site_replicate_id": replicate_id,
        "temperature_of_isolation": 0,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: A1",
        "dna_id": dna_id
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/isolates")
        .header("Content-Type", "application/json")
        .body(Body::from(create_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let isolate: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let isolate_id = isolate.get("id").unwrap().as_str().unwrap().to_string();

    // === READ ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/isolates/{}", isolate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(fetched.get("name").unwrap().as_str().unwrap(), "A");

    // === UPDATE ===
    let update_payload = json!({
        "name": "A_Updated",
        "taxonomy": "Pseudomonas Updated",
        "photo": "",
        "site_replicate_id": replicate_id,
        "temperature_of_isolation": 0,
        "media_used_for_isolation": "M9",
        "storage_location": "Isolates: A1",
        "dna_id": dna_id
    });
    let request = Request::builder()
        .method("PUT")
        .uri(format!("/api/isolates/{}", isolate_id))
        .header("Content-Type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(updated.get("name").unwrap().as_str().unwrap(), "A_Updated");

    // === DELETE ===
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/isolates/{}", isolate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // === VERIFY DELETION ===
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/isolates/{}", isolate_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
