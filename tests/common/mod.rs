use axum::Router;
use cryobiobank_api::{
    common::views as common_views, config::Config, dna::views as dna_views,
    isolates::views as iso_views, samples::views as samp_views,
    sites::replicates::views as sr_views, sites::views as sites_views,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

/// Connects to the test database, drops and recreates the public schema,
/// then runs all migrations so that every test starts from a known state.
pub async fn setup_clean_db() -> DatabaseConnection {
    let config = Config::from_env();
    let db_url = config.db_url.as_ref().expect("db_url not set");
    let db = Database::connect(db_url)
        .await
        .expect("Failed to connect to test DB");

    // Clean the DB: drop and recreate the public schema.
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

    // Run migrations.
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

/// Builds an Axum router that nests all your endpoints.
pub fn build_app_with_db(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/healthz", axum::routing::get(common_views::healthz))
        .route(
            "/api/config",
            axum::routing::get(common_views::get_ui_config),
        )
        .with_state(db.clone())
        .nest("/api/sites", sites_views::router(db.clone(), None))
        .nest("/api/site_replicates", sr_views::router(db.clone(), None))
        .nest("/api/dna", dna_views::router(db.clone(), None))
        .nest("/api/isolates", iso_views::router(db.clone(), None))
        .nest("/api/samples", samp_views::router(db.clone(), None))
}
