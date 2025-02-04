// tests/crud_tests.rs

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use cryobiobank_api::{
    common::views as common_views,
    config::Config,
    sites::{models::Site, views},
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use serde_json::json;
use tower::ServiceExt;

/// Sets up a clean database by dropping the public schema (removing all data),
/// recreating it, and then running all migrations.
async fn setup_clean_db() -> DatabaseConnection {
    let config = Config::from_env();
    let db_url = config.db_url.as_ref().expect("db_url not set");
    let db = Database::connect(db_url)
        .await
        .expect("Failed to connect to test DB");

    // Drop and recreate public schema so we start with no data.
    let drop_query = Statement::from_string(
        DbBackend::Postgres,
        "DROP SCHEMA public CASCADE;".to_owned(),
    );
    db.execute(drop_query)
        .await
        .expect("Failed to drop public schema");
    let create_query =
        Statement::from_string(DbBackend::Postgres, "CREATE SCHEMA public;".to_owned());
    db.execute(create_query)
        .await
        .expect("Failed to create public schema");

    // Run migrations (this will also re-create any tables needed).
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations in test DB");

    db
}

/// Builds an Axum router with the routes under test. In this example we only include
/// the /healthz, /api/config and /api/sites routes. You should add additional nestings
/// (e.g. for site_replicates, dna, isolates, samples) in a similar fashion.
fn build_app_with_db(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/healthz", axum::routing::get(common_views::healthz))
        .route(
            "/api/config",
            axum::routing::get(common_views::get_ui_config),
        )
        .with_state(db.clone())
        .nest("/api/sites", views::router(db.clone(), None))
    // Extend here with additional endpoints, for example:
    // .nest("/api/site_replicates", site_replicates_router(db.clone(), None))
    // .nest("/api/dna", dna_router(db.clone(), None))
    // .nest("/api/isolates", isolates_router(db.clone(), None))
    // .nest("/api/samples", samples_router(db.clone(), None))
}

/// This test performs a complete CRUD cycle on the /api/sites endpoint.
/// (You’d write similar tests for the other endpoints, ensuring that any foreign-key
/// constraints are met by creating parent records first.)
#[tokio::test]
async fn crud_sites() {
    // Setup a clean DB and build our application router.
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
