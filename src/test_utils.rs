use crate::{
    areas::db::Area as area_views, common::views as common_views, config::Config,
    dna::db::DNA as dna_views, field_records::db::FieldRecord as fr_views,
    isolates::db::Isolate as iso_views, middleware, samples::db::Sample as samp_views,
    sites::db::Site as sites_views,
};
use axum::Router;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema, Statement};

pub async fn setup_clean_db() -> DatabaseConnection {
    let config = Config::from_env();
    let db_url = config.db_url.as_ref().expect("db_url not set");
    let db = Database::connect(db_url)
        .await
        .expect("Failed to connect to test DB");

    let truncate_query = Statement::from_string(
        DbBackend::Postgres,
        "TRUNCATE TABLE samples, isolates, field_records, dna, sites, areas RESTART IDENTITY CASCADE;"
            .to_owned(),
    );
    db.execute(truncate_query).await.unwrap();
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

pub async fn setup_sqlite_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    let tables: Vec<sea_orm::sea_query::TableCreateStatement> = vec![
        schema.create_table_from_entity(crate::areas::db::Entity),
        schema.create_table_from_entity(crate::sites::db::Entity),
        schema.create_table_from_entity(crate::field_records::db::Entity),
        schema.create_table_from_entity(crate::samples::db::Entity),
        schema.create_table_from_entity(crate::isolates::db::Entity),
        schema.create_table_from_entity(crate::dna::db::Entity),
    ];

    for stmt in tables {
        db.execute(backend.build(&stmt))
            .await
            .expect("Failed to create table");
    }

    db
}

pub fn build_app_with_db(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/healthz", axum::routing::get(common_views::healthz))
        .route(
            "/api/config",
            axum::routing::get(common_views::get_ui_config),
        )
        .with_state(db.clone())
        .nest("/api/sites", sites_views::router(&db).split_for_parts().0)
        .nest(
            "/api/field_records",
            fr_views::router(&db).split_for_parts().0,
        )
        .nest("/api/dna", dna_views::router(&db).split_for_parts().0)
        .nest("/api/isolates", iso_views::router(&db).split_for_parts().0)
        .nest("/api/samples", samp_views::router(&db).split_for_parts().0)
        .nest("/api/areas", area_views::router(&db).split_for_parts().0)
}

/// Build app with scope middleware applied (simulates unauthenticated public access).
/// No keycloak layer — ScopeCondition is always injected on every request.
pub fn build_scoped_app_with_db(db: DatabaseConnection) -> Router {
    Router::new()
        .with_state(db.clone())
        .nest(
            "/api/sites",
            sites_views::router(&db)
                .split_for_parts()
                .0
                .layer(axum::middleware::from_fn(middleware::scope_sites)),
        )
        .nest(
            "/api/field_records",
            fr_views::router(&db)
                .split_for_parts()
                .0
                .layer(axum::middleware::from_fn(middleware::scope_field_records)),
        )
        .nest(
            "/api/dna",
            dna_views::router(&db)
                .split_for_parts()
                .0
                .layer(axum::middleware::from_fn(middleware::scope_dna)),
        )
        .nest(
            "/api/isolates",
            iso_views::router(&db)
                .split_for_parts()
                .0
                .layer(axum::middleware::from_fn(middleware::scope_isolates)),
        )
        .nest(
            "/api/samples",
            samp_views::router(&db)
                .split_for_parts()
                .0
                .layer(axum::middleware::from_fn(middleware::scope_samples)),
        )
        .nest(
            "/api/areas",
            area_views::router(&db)
                .split_for_parts()
                .0
                .layer(axum::middleware::from_fn(middleware::scope_areas)),
        )
}
