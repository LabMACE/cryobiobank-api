use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::test_utils::{build_app_with_db, setup_clean_db, setup_sqlite_db};

async fn post_json(
    app: &axum::Router,
    uri: &str,
    payload: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(json!(null));
    (status, json)
}

async fn get_one(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(json!(null));
    (status, json)
}

async fn create_site(app: &axum::Router) -> String {
    let (status, site) = post_json(
        app,
        "/api/sites",
        json!({
            "name": "Test Site",
            "latitude_4326": 46.5,
            "longitude_4326": 7.3,
            "elevation_metres": 1500.0
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    site["id"].as_str().unwrap().to_string()
}

async fn create_field_record(app: &axum::Router, site_id: &str) -> String {
    let (status, fr) = post_json(
        app,
        "/api/field_records",
        json!({
            "site_id": site_id,
            "name": "FR-001",
            "sample_type": "Snow",
            "sampling_date": "2024-03-15"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    fr["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_get_site() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;

    let (status, site) = get_one(&app, &format!("/api/sites/{site_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(site["name"].as_str().unwrap(), "Test Site");
    assert_eq!(site["latitude_4326"].as_f64().unwrap(), 46.5);
    assert_eq!(site["longitude_4326"].as_f64().unwrap(), 7.3);
    assert_eq!(site["elevation_metres"].as_f64().unwrap(), 1500.0);
}

#[tokio::test]
async fn create_and_get_field_record() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;
    let fr_id = create_field_record(&app, &site_id).await;

    let (status, fr) = get_one(&app, &format!("/api/field_records/{fr_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fr["name"].as_str().unwrap(), "FR-001");
    assert_eq!(fr["sample_type"].as_str().unwrap(), "Snow");
    assert_eq!(fr["site_id"].as_str().unwrap(), site_id);
}

#[tokio::test]
async fn create_and_get_sample() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;
    let fr_id = create_field_record(&app, &site_id).await;

    let (status, sample) = post_json(
        &app,
        "/api/samples",
        json!({
            "field_record_id": fr_id,
            "name": "Sample-001",
            "description": "Snow sample from glacier",
            "storage_location": "Freezer A"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let sample_id = sample["id"].as_str().unwrap();
    let (status, sample) = get_one(&app, &format!("/api/samples/{sample_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sample["name"].as_str().unwrap(), "Sample-001");
    assert_eq!(sample["field_record_id"].as_str().unwrap(), fr_id);
    assert!(sample["is_available"].as_bool().unwrap());
}

#[tokio::test]
async fn create_isolate_without_photo() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;
    let fr_id = create_field_record(&app, &site_id).await;

    let (status, isolate) = post_json(
        &app,
        "/api/isolates",
        json!({
            "field_record_id": fr_id,
            "name": "Isolate-001",
            "taxonomy": "Pseudomonas",
            "temperature_of_isolation": 4.0,
            "media_used_for_isolation": "R2A"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let isolate_id = isolate["id"].as_str().unwrap();
    let (status, isolate) = get_one(&app, &format!("/api/isolates/{isolate_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(isolate["name"].as_str().unwrap(), "Isolate-001");
    assert_eq!(isolate["taxonomy"].as_str().unwrap(), "Pseudomonas");
    assert!(isolate["photo"].is_null() || isolate["photo"].as_str() == Some(""));
}

#[tokio::test]
async fn create_isolate_with_photo() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;
    let fr_id = create_field_record(&app, &site_id).await;

    let (status, isolate) = post_json(
        &app,
        "/api/isolates",
        json!({
            "field_record_id": fr_id,
            "name": "Isolate-WithPhoto",
            "taxonomy": "Bacillus",
            "photo": "data:image/png;base64,iVBOR..."
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let isolate_id = isolate["id"].as_str().unwrap();
    let (status, isolate) = get_one(&app, &format!("/api/isolates/{isolate_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        isolate["photo"].as_str().unwrap(),
        "data:image/png;base64,iVBOR..."
    );
}

#[tokio::test]
async fn create_and_get_dna() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;
    let fr_id = create_field_record(&app, &site_id).await;

    let (status, dna) = post_json(
        &app,
        "/api/dna",
        json!({
            "field_record_id": fr_id,
            "name": "DNA-001",
            "description": "16S rRNA extraction",
            "extraction_method": "PowerSoil Kit"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let dna_id = dna["id"].as_str().unwrap();
    let (status, dna) = get_one(&app, &format!("/api/dna/{dna_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(dna["name"].as_str().unwrap(), "DNA-001");
    assert_eq!(dna["extraction_method"].as_str().unwrap(), "PowerSoil Kit");
    assert_eq!(dna["field_record_id"].as_str().unwrap(), fr_id);
}

#[tokio::test]
async fn full_hierarchy_creation() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let site_id = create_site(&app).await;
    let fr_id = create_field_record(&app, &site_id).await;

    let (s1, _) = post_json(
        &app,
        "/api/samples",
        json!({ "field_record_id": fr_id, "name": "S1" }),
    )
    .await;
    let (s2, _) = post_json(
        &app,
        "/api/isolates",
        json!({ "field_record_id": fr_id, "name": "I1", "taxonomy": "Unknown" }),
    )
    .await;
    let (s3, _) = post_json(
        &app,
        "/api/dna",
        json!({ "field_record_id": fr_id, "name": "D1" }),
    )
    .await;

    assert_eq!(s1, StatusCode::CREATED);
    assert_eq!(s2, StatusCode::CREATED);
    assert_eq!(s3, StatusCode::CREATED);
}

#[tokio::test]
async fn reject_invalid_isolate() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let (status, _) = post_json(
        &app,
        "/api/isolates",
        json!({
            "name": "Bad Isolate",
            "field_record_id": "not-a-uuid"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn reject_missing_required_fields() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let (status, _) = post_json(&app, "/api/sites", json!({ "name": "Missing coords" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// --- Postgres-backed ingestion workflow (e2e stack) -------------------------
// These run against the real PostGIS test DB (`#[ignore]`d so they don't run on
// a plain `cargo test` without one). They cover what the in-memory SQLite
// bulk_import_tests can't: the list endpoints whose handlers use PostGIS /
// array-join SQL (areas convex hull, field_records & isolates joins), real
// foreign-key enforcement, and the sample_type enum. Mirrors the wizard, which
// resolves names to ids client-side and posts ID-based rows to /batch.

async fn batch(app: &axum::Router, entity: &str, payload: serde_json::Value) -> (StatusCode, serde_json::Value) {
    post_json(app, &format!("/api/{entity}/batch?partial=true"), payload).await
}

async fn list(app: &axum::Router, entity: &str) -> Vec<serde_json::Value> {
    let uri = format!(
        "/api/{entity}?sort=%5B%22name%22%2C%22ASC%22%5D&range=%5B0%2C99%5D&filter=%7B%7D"
    );
    let (status, body) = get_one(app, &uri).await;
    assert_eq!(status, StatusCode::OK, "list {entity}: {body}");
    body.as_array().expect("list response is an array").clone()
}

#[tokio::test]
#[ignore]
async fn pg_full_hierarchy_then_lists() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db);

    let (s, areas) = batch(&app, "areas", json!([{ "name": "PG Alps", "colour": "#1565c0" }])).await;
    assert_eq!(s, StatusCode::CREATED, "areas: {areas}");
    let area_id = areas["succeeded"][0]["id"].as_str().unwrap();

    let (s, sites) = batch(
        &app,
        "sites",
        json!([{ "name": "PG Glacier", "latitude_4326": 46.5, "longitude_4326": 7.3, "elevation_metres": 3200.0, "area_id": area_id }]),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "sites: {sites}");
    let site_id = sites["succeeded"][0]["id"].as_str().unwrap();

    let (s, frs) = batch(
        &app,
        "field_records",
        json!([{ "site_id": site_id, "name": "PG-FR-001", "sample_type": "Snow", "sampling_date": "2025-03-15" }]),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "field_records: {frs}");
    let fr_id = frs["succeeded"][0]["id"].as_str().unwrap();

    let (s, _) = batch(&app, "samples", json!([{ "field_record_id": fr_id, "name": "PG-S-1" }])).await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, _) = batch(
        &app,
        "isolates",
        json!([{ "field_record_id": fr_id, "name": "PG-ISO-1", "taxonomy": "Pseudomonas sp." }]),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, _) = batch(&app, "dna", json!([{ "field_record_id": fr_id, "name": "PG-DNA-1" }])).await;
    assert_eq!(s, StatusCode::CREATED);

    // The PostGIS / array-join list handlers (only runnable on Postgres).
    assert_eq!(list(&app, "areas").await.len(), 1);
    assert_eq!(list(&app, "sites").await.len(), 1);
    assert_eq!(list(&app, "field_records").await.len(), 1);
    assert_eq!(list(&app, "samples").await.len(), 1);
    assert_eq!(list(&app, "isolates").await.len(), 1);
    assert_eq!(list(&app, "dna").await.len(), 1);
}

#[tokio::test]
#[ignore]
async fn pg_partial_batch_reports_fk_violation() {
    // A sample referencing a non-existent field record violates the FK. With
    // partial mode and a single bad row, the whole request is a 400 but the body
    // still carries the per-row failure (Postgres enforces the FK; SQLite doesn't).
    let db = setup_clean_db().await;
    let app = build_app_with_db(db);

    let (status, body) = batch(
        &app,
        "samples",
        json!([{ "field_record_id": "00000000-0000-0000-0000-0000000000ff", "name": "Orphan" }]),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "fk violation: {body}");
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert!(!failed[0]["error"].as_str().unwrap().is_empty());
}

#[tokio::test]
#[ignore]
async fn pg_invalid_sample_type_enum_rejected() {
    // sample_type is a Rust/Postgres enum (Snow|Soil); an unknown variant fails
    // body deserialization for the whole batch → 422.
    let db = setup_clean_db().await;
    let app = build_app_with_db(db);

    let (s, sites) = batch(
        &app,
        "sites",
        json!([{ "name": "Enum Site", "latitude_4326": 46.0, "longitude_4326": 7.0, "elevation_metres": 1000.0 }]),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let site_id = sites["succeeded"][0]["id"].as_str().unwrap();

    let (status, _) = post_json(
        &app,
        "/api/field_records/batch?partial=true",
        json!([{ "site_id": site_id, "name": "Bad Enum FR", "sample_type": "Ice", "sampling_date": "2025-03-15" }]),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
