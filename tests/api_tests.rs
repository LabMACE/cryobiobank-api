// tests/api_tests.rs

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
    // ServiceExt needed for .oneshot
};
use axum_keycloak_auth::{
    instance::{KeycloakAuthInstance, KeycloakConfig},
    Url,
};
use tower::ServiceExt;

use sea_orm::{ActiveValue::Set, Database, DatabaseConnection, EntityTrait};
use uuid::Uuid;

// Bring in your migration system (the same one used in main.rs).
use migration::{Migrator, MigratorTrait};

// Bring in your config so we can read DB credentials from env.
use cryobiobank_api::config::Config;

// Bring in the modules needed for seeding and testing.
use cryobiobank_api::common::views as common_views;
use cryobiobank_api::sites::db; // The Entity
use cryobiobank_api::sites::models::Site;
use cryobiobank_api::sites::views; // The router
                                   // ... add more imports if needed for other endpoints

/// A helper function to:
/// 1) Read DB connection info from env (using your Config).
/// 2) Connect to the test DB.
/// 3) Run migrations (so your schema matches).
async fn setup_test_db() -> DatabaseConnection {
    // Read standard config from environment variables (like in main.rs).
    let config = Config::from_env();

    // Build the DB URL, e.g. "postgresql://postgres:psql@cryobiobank-test-db:5432/postgres"
    // The .unwrap() below is OK for test code but handle errors as you wish.
    let db_url = config.db_url.as_ref().unwrap();

    // Connect to the test DB.
    let db = Database::connect(db_url)
        .await
        .expect("Failed to connect to test DB");

    // Run migrations (same as in main.rs).
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations in test DB");

    db
}

/// A helper function to insert seed data into the DB.
/// This example seeds one "Test Site".
async fn seed_sites(db: &DatabaseConnection) {
    let test_site = db::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Test Site".to_owned()),
        // x: Set(Some(1.23)),
        // y: Set(Some(4.56)),
        // z: Set(Some(7.89)),
        ..Default::default()
    };

    db::Entity::insert(test_site)
        .exec(db)
        .await
        .expect("Failed to insert test site");
}

/// A helper function to build the Axum router (like `main.rs`) but for testing only.
/// If you want to test multiple endpoints, nest them all here the same way you do in main.rs.
/// This example just includes the sites router. Adjust as needed.
fn build_test_app(db: DatabaseConnection) -> Router {
    // Create a dummy keycloak auth instance.
    // (If you need real tokens, see the keycloak docs or your own test approach.)
    let keycloak_auth_instance: Arc<KeycloakAuthInstance> = Arc::new(KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse("http://fake-keycloak-for-tests").unwrap())
            .realm("testrealm".to_owned())
            .build(),
    ));

    // Build your full router, or just the portion you want to test.
    // This is the same code you use in main.rs, with .nest(...) for each endpoint group.
    Router::new()
        .route("/healthz", axum::routing::get(common_views::healthz))
        .route(
            "/api/config",
            axum::routing::get(common_views::get_ui_config),
        )
        .with_state(db.clone())
        .nest(
            "/api/sites",
            views::router(db.clone(), keycloak_auth_instance.clone()),
        )
    // .nest("/api/isolates", isolates::views::router(...))  // example
    // .nest("/api/samples", samples::views::router(...))    // example
}

/// Actual test which checks "GET /api/sites".
#[tokio::test]
async fn test_get_all_sites() {
    // 1) Setup the test DB and run migrations.
    let db = setup_test_db().await;

    // 2) Insert test data (one site).
    seed_sites(&db).await;

    // 3) Build the router (as in main.rs, but test-friendly).
    let app = build_test_app(db.clone());

    // 4) Send a GET request to `/api/sites/?range=0-10`.
    //    Notice the path is `/?range=0-10` if your router is at "/api/sites"
    //    then we need the final path to be "/api/sites/?range=0-10".
    //    In other words, the path we pass to `Request::builder()` must match
    //    how you've set up your route in `views::router`.
    let request = Request::builder()
        .uri("/?range=0-10")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // 5) Assert we got status 200 OK.
    assert_eq!(response.status(), StatusCode::OK);

    // 6) Parse the body as JSON array of Sites.
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("Failed to read response body");
    let sites: Vec<Site> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse JSON response into Vec<Site>");

    // 7) Assert we have at least one site.
    assert!(!sites.is_empty(), "Expected at least one site in response");

    // 8) Check the site data is what we inserted.
    let first_site = &sites[0];
    assert_eq!(first_site.name, "Test Site");
    assert_eq!(first_site.longitude_4326, Some(1.23));
    assert_eq!(first_site.latitude_4326, Some(4.56));
    assert_eq!(first_site.elevation_metres, Some(7.89));
}

// If you want to do additional tests (e.g., GET /api/sites/{id}), add them here:
// #[tokio::test]
// async fn test_get_one_site() { ... }
