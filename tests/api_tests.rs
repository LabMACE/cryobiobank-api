use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use cryobiobank_api::common::views as common_views;
use cryobiobank_api::config::Config;
use cryobiobank_api::sites::db;
use cryobiobank_api::sites::models::Site;
use cryobiobank_api::sites::views;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveValue::Set, ConnectionTrait, Database, DatabaseConnection, DbBackend, EntityTrait,
    Statement,
};
use tower::ServiceExt;
use uuid::Uuid;

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
async fn tear_down_test_db(db: &DatabaseConnection) {
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
}

/// A helper function to insert seed data into the DB.
/// This example seeds one "Test Site".
async fn seed_sites(db: &DatabaseConnection) {
    let test_site = db::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Test Site".to_owned()),
        longitude_4326: Set(1.23),
        latitude_4326: Set(4.56),
        elevation_metres: Set(7.89),
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
    Router::new()
        .route("/healthz", axum::routing::get(common_views::healthz))
        .route(
            "/api/config",
            axum::routing::get(common_views::get_ui_config),
        )
        .with_state(db.clone())
        .nest("/api/sites", views::router(db.clone(), None))
}
#[tokio::test]
async fn test_get_all_sites() {
    let db = setup_test_db().await;
    seed_sites(&db).await;
    let app = build_test_app(db.clone());
    let request = Request::builder()
        .uri("/api/sites")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("Failed to read response body");
    let sites: Vec<Site> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse JSON response into Vec<Site>");

    assert!(!sites.is_empty(), "Expected at least one site in response");

    let first_site = &sites[0];
    assert_eq!(first_site.name, "Test Site");
    assert_eq!(first_site.longitude_4326, 1.23);
    assert_eq!(first_site.latitude_4326, 4.56);
    assert_eq!(first_site.elevation_metres, 7.89);

    // 9) Tear down the test DB so that the next test starts fresh.
    tear_down_test_db(&db).await;
}
