use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::test_utils::{build_app_with_db, setup_sqlite_db};

async fn post_json(app: &axum::Router, uri: &str, payload: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!(null));
    (status, json)
}

/// Mirror the UI import wizard: POST to `/batch?partial=true` and read the
/// `succeeded` array out of the `BatchResult`. Asserts the whole batch landed.
async fn batch_create(app: &axum::Router, entity: &str, payload: Value) -> Vec<Value> {
    let (status, json) = post_json(app, &format!("/api/{entity}/batch?partial=true"), payload).await;
    assert_eq!(status, StatusCode::CREATED, "batch create {entity}: {json}");
    assert!(
        json["failed"].as_array().unwrap().is_empty(),
        "unexpected failures creating {entity}: {json}"
    );
    json["succeeded"].as_array().unwrap().clone()
}

async fn get_one(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!(null));
    (status, json)
}

/// List endpoint for entities whose `get_all` uses plain queries (sites, samples,
/// dna). Areas, field_records and isolates run array/PostGIS queries in their
/// list handlers that SQLite can't execute, so those are verified by id instead.
async fn get_list(app: &axum::Router, entity: &str) -> Vec<Value> {
    let uri = format!("/api/{entity}?sort=%5B%22name%22%2C%22ASC%22%5D&range=%5B0%2C99%5D&filter=%7B%7D");
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Range", format!("{entity}=0-99"))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "list {entity}");
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    json.as_array().unwrap().clone()
}

#[tokio::test]
async fn bulk_import_full_hierarchy() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    // Areas
    let areas = batch_create(
        &app,
        "areas",
        json!([
            { "name": "Alps West", "colour": "#ff0000" },
            { "name": "Alps East", "colour": "#0000ff", "description": "Eastern alpine region" }
        ]),
    )
    .await;
    assert_eq!(areas.len(), 2);
    let area_west_id = areas[0]["id"].as_str().unwrap();
    let area_east_id = areas[1]["id"].as_str().unwrap();

    // Sites across both areas (the FK is resolved by the wizard from area name)
    let sites = batch_create(
        &app,
        "sites",
        json!([
            { "name": "Glacier Peak", "latitude_4326": 46.5, "longitude_4326": 7.3, "elevation_metres": 3200.0, "area_id": area_west_id },
            { "name": "Snow Basin", "latitude_4326": 46.8, "longitude_4326": 7.5, "elevation_metres": 2800.0, "area_id": area_west_id },
            { "name": "Eastern Ridge", "latitude_4326": 47.0, "longitude_4326": 10.2, "elevation_metres": 2500.0, "area_id": area_east_id }
        ]),
    )
    .await;
    assert_eq!(sites.len(), 3);
    let site_glacier = sites[0]["id"].as_str().unwrap();
    let site_snow = sites[1]["id"].as_str().unwrap();
    let site_eastern = sites[2]["id"].as_str().unwrap();

    // Field records spanning multiple sites, with measurement data
    let frs = batch_create(
        &app,
        "field_records",
        json!([
            {
                "site_id": site_glacier, "name": "FR-GP-001", "sample_type": "Snow",
                "sampling_date": "2025-03-15", "snow_depth_cm": 120.5,
                "air_temperature_celsius": -8.0, "ph": 6.2, "ions_chloride": 0.15, "ions_nitrate": 0.08
            },
            {
                "site_id": site_glacier, "name": "FR-GP-002", "sample_type": "Soil",
                "sampling_date": "2025-07-20", "sample_depth_cm": 10.0,
                "air_temperature_celsius": 12.5, "ph": 5.8,
                "organic_acids_formate": 0.02, "organic_acids_acetate": 0.05
            },
            { "site_id": site_snow, "name": "FR-SB-001", "sample_type": "Snow", "sampling_date": "2025-02-10" },
            {
                "site_id": site_eastern, "name": "FR-ER-001", "sample_type": "Soil",
                "sampling_date": "2025-08-01", "bacterial_abundance": 1500000, "cfu_count_r2a": 250
            }
        ]),
    )
    .await;
    assert_eq!(frs.len(), 4);
    let fr_gp1 = frs[0]["id"].as_str().unwrap();
    let fr_gp2 = frs[1]["id"].as_str().unwrap();
    let fr_sb1 = frs[2]["id"].as_str().unwrap();
    let fr_er1 = frs[3]["id"].as_str().unwrap();

    // Measurement fields round-tripped through the batch create
    assert_eq!(frs[0]["snow_depth_cm"].as_f64().unwrap(), 120.5);
    assert_eq!(frs[0]["ions_chloride"].as_f64().unwrap(), 0.15);
    assert_eq!(frs[1]["organic_acids_acetate"].as_f64().unwrap(), 0.05);
    assert_eq!(frs[3]["bacterial_abundance"].as_i64().unwrap(), 1500000);

    // Samples
    let samples = batch_create(
        &app,
        "samples",
        json!([
            { "field_record_id": fr_gp1, "name": "S-GP1-A", "description": "Surface snow" },
            { "field_record_id": fr_gp1, "name": "S-GP1-B", "storage_location": "Freezer B2" },
            { "field_record_id": fr_sb1, "name": "S-SB1-A" }
        ]),
    )
    .await;
    assert_eq!(samples.len(), 3);
    assert!(samples[0]["is_available"].as_bool().unwrap());

    // Isolates (no photos, as per import scope)
    let isolates = batch_create(
        &app,
        "isolates",
        json!([
            { "field_record_id": fr_gp2, "name": "ISO-GP2-001", "taxonomy": "Pseudomonas sp.", "temperature_of_isolation": 4.0, "media_used_for_isolation": "R2A" },
            { "field_record_id": fr_er1, "name": "ISO-ER1-001", "taxonomy": "Bacillus sp." }
        ]),
    )
    .await;
    assert_eq!(isolates.len(), 2);

    // DNA
    let dna = batch_create(
        &app,
        "dna",
        json!([
            { "field_record_id": fr_gp1, "name": "DNA-GP1-16S", "extraction_method": "PowerSoil Kit", "description": "16S rRNA amplicon" },
            { "field_record_id": fr_gp2, "name": "DNA-GP2-META", "extraction_method": "DNeasy" }
        ]),
    )
    .await;
    assert_eq!(dna.len(), 2);

    // === Verify persisted state ===

    // Counts via list endpoints that SQLite can run.
    assert_eq!(get_list(&app, "sites").await.len(), 3);
    assert_eq!(get_list(&app, "samples").await.len(), 3);
    assert_eq!(get_list(&app, "dna").await.len(), 2);

    // Areas (list uses convex-hull array query — verify by id instead).
    let (status, a) = get_one(&app, &format!("/api/areas/{area_west_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(a["name"].as_str().unwrap(), "Alps West");
    let (status, a) = get_one(&app, &format!("/api/areas/{area_east_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(a["description"].as_str().unwrap(), "Eastern alpine region");

    // Sites — FK to areas resolved
    let (status, s) = get_one(&app, &format!("/api/sites/{site_glacier}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(s["area_id"].as_str().unwrap(), area_west_id);
    assert_eq!(s["elevation_metres"].as_f64().unwrap(), 3200.0);
    let (status, s) = get_one(&app, &format!("/api/sites/{site_eastern}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(s["area_id"].as_str().unwrap(), area_east_id);

    // Field records (list uses join array query — verify by id) — FK + measurements
    let (status, fr) = get_one(&app, &format!("/api/field_records/{fr_gp1}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fr["site_id"].as_str().unwrap(), site_glacier);
    assert_eq!(fr["sample_type"].as_str().unwrap(), "Snow");
    assert_eq!(fr["snow_depth_cm"].as_f64().unwrap(), 120.5);
    let (status, fr) = get_one(&app, &format!("/api/field_records/{fr_er1}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fr["bacterial_abundance"].as_i64().unwrap(), 1500000);

    // Samples — FK + default is_available
    let sample_id = samples[0]["id"].as_str().unwrap();
    let (status, s) = get_one(&app, &format!("/api/samples/{sample_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(s["field_record_id"].as_str().unwrap(), fr_gp1);
    assert!(s["is_available"].as_bool().unwrap());

    // Isolates (list uses photo-flag array query — verify by id) — FK + taxonomy
    let iso_id = isolates[0]["id"].as_str().unwrap();
    let (status, iso) = get_one(&app, &format!("/api/isolates/{iso_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(iso["taxonomy"].as_str().unwrap(), "Pseudomonas sp.");
    assert_eq!(iso["field_record_id"].as_str().unwrap(), fr_gp2);

    // DNA — FK
    let dna_id = dna[0]["id"].as_str().unwrap();
    let (status, d) = get_one(&app, &format!("/api/dna/{dna_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(d["extraction_method"].as_str().unwrap(), "PowerSoil Kit");
    assert_eq!(d["field_record_id"].as_str().unwrap(), fr_gp1);
}

#[tokio::test]
async fn batch_create_rejects_duplicate_name() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let (status, _) = post_json(
        &app,
        "/api/areas/batch",
        json!([{ "name": "Unique Area", "colour": "#123456" }]),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Default (all-or-nothing) mode: a clashing name fails the whole batch —
    // crudcrate maps the unique-constraint violation to a 409 Conflict, so this
    // must not report success.
    let (status, _) = post_json(
        &app,
        "/api/areas/batch",
        json!([{ "name": "Unique Area", "colour": "#654321" }]),
    )
    .await;
    assert_ne!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn batch_create_partial_mode_reports_failed_rows() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let (status, _) = post_json(
        &app,
        "/api/areas/batch",
        json!([{ "name": "Existing Area", "colour": "#111111" }]),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // One new + one duplicate: the new row commits, the duplicate is reported.
    let (status, json) = post_json(
        &app,
        "/api/areas/batch?partial=true",
        json!([
            { "name": "New Area", "colour": "#222222" },
            { "name": "Existing Area", "colour": "#333333" }
        ]),
    )
    .await;

    assert_eq!(status, StatusCode::MULTI_STATUS);
    let succeeded = json["succeeded"].as_array().unwrap();
    let failed = json["failed"].as_array().unwrap();
    assert_eq!(succeeded.len(), 1);
    assert_eq!(succeeded[0]["name"].as_str().unwrap(), "New Area");
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["index"].as_u64().unwrap(), 1);
    // The failed row carries crudcrate's per-row "... already exists" message. The
    // import wizard also catches duplicates earlier (preview-time "Already exists
    // in the database").
    assert!(!failed[0]["error"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn batch_create_partial_mode_all_failed_returns_400() {
    // Backs the submit hook's catch path: when every row in a chunk is rejected
    // the request is a 400, but the body still carries one failure per row.
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let _ = batch_create(
        &app,
        "areas",
        json!([
            { "name": "Dup A", "colour": "#111111" },
            { "name": "Dup B", "colour": "#222222" }
        ]),
    )
    .await;

    let (status, json) = post_json(
        &app,
        "/api/areas/batch?partial=true",
        json!([
            { "name": "Dup A", "colour": "#333333" },
            { "name": "Dup B", "colour": "#444444" }
        ]),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json["succeeded"].as_array().unwrap().is_empty());
    let failed = json["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 2);
    assert_eq!(failed[0]["index"].as_u64().unwrap(), 0);
    assert_eq!(failed[1]["index"].as_u64().unwrap(), 1);
    assert!(!failed[0]["error"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn batch_create_rejects_over_limit() {
    // The wizard chunks at 100 because the API caps a single batch at that size.
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let payload: Vec<Value> = (0..101)
        .map(|i| json!({ "name": format!("Area {i}"), "colour": "#000000" }))
        .collect();

    let (status, _) = post_json(&app, "/api/areas/batch", json!(payload)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ----------------------------------------------------------------------------
// New field record and DNA columns (July 2026).
// ----------------------------------------------------------------------------

/// Create one area + site + field record and return the field-record id, so the
/// new-field tests below can hang a record off a valid site.
async fn seed_site_field_record(app: &axum::Router, fr_payload: Value) -> String {
    let areas = batch_create(
        app,
        "areas",
        json!([{ "name": "Seed Area", "colour": "#123456" }]),
    )
    .await;
    let area_id = areas[0]["id"].as_str().unwrap();
    let sites = batch_create(
        app,
        "sites",
        json!([{ "name": "Seed Site", "latitude_4326": 46.5, "longitude_4326": 7.3, "elevation_metres": 2000.0, "area_id": area_id }]),
    )
    .await;
    let site_id = sites[0]["id"].as_str().unwrap();

    let mut payload = fr_payload;
    payload["site_id"] = json!(site_id);
    let frs = batch_create(app, "field_records", json!([payload])).await;
    frs[0]["id"].as_str().unwrap().to_string()
}

/// Scenario: reorganised field records gain Treatment, Campaign, Water content,
/// and the elemental-content trio (TC / TOC / TN).
/// Expected behaviour: each value is persisted and returned on read.
#[tokio::test]
async fn field_record_accepts_new_fields() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let fr_id = seed_site_field_record(
        &app,
        json!({
            "name": "FR-NEW-001",
            "sample_type": "Soil",
            "sampling_date": "2026-06-01",
            "treatment": "control",
            "campaign": "Summer 2026",
            "water_content": 34.2,
            "total_carbon": 5.1,
            "total_organic_carbon": 4.7,
            "total_nitrogen": 0.42
        }),
    )
    .await;

    let (status, fr) = get_one(&app, &format!("/api/field_records/{fr_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fr["treatment"], json!("control"));
    assert_eq!(fr["campaign"], json!("Summer 2026"));
    assert_eq!(fr["water_content"], json!(34.2));
    assert_eq!(fr["total_carbon"], json!(5.1));
    assert_eq!(fr["total_organic_carbon"], json!(4.7));
    assert_eq!(fr["total_nitrogen"], json!(0.42));
}

/// Scenario: DNA records gain Volume and Concentration. The metagenome URL is
/// not stored on DNA; it lives on the parent field record and DNA displays it
/// (inherited), so it is intentionally absent from the DNA payload.
/// Expected behaviour: volume and concentration are persisted and returned on read.
#[tokio::test]
async fn dna_accepts_volume_and_concentration() {
    let db = setup_sqlite_db().await;
    let app = build_app_with_db(db);

    let fr_id = seed_site_field_record(
        &app,
        json!({ "name": "FR-DNA-001", "sample_type": "Soil", "sampling_date": "2026-06-01" }),
    )
    .await;

    let dna = batch_create(
        &app,
        "dna",
        json!([{
            "name": "DNA-NEW-001",
            "field_record_id": fr_id,
            "volume": 25.0,
            "concentration": 12.8
        }]),
    )
    .await;
    let dna_id = dna[0]["id"].as_str().unwrap();

    let (status, d) = get_one(&app, &format!("/api/dna/{dna_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(d["volume"], json!(25.0));
    assert_eq!(d["concentration"], json!(12.8));
}
