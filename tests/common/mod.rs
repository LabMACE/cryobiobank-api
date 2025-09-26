use axum::Router;
use cryobiobank_api::{
    common::views as common_views, config::Config, dna::db::DNA as dna_views,
    isolates::db::Isolate as iso_views, samples::db::Sample as samp_views,
    sites::db::Site as sites_views, sites::replicates::db::SiteReplicate as sr_views,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn setup_clean_db() -> DatabaseConnection {
    let config = Config::from_env();
    let db_url = config.db_url.as_ref().expect("db_url not set");
    let db = Database::connect(db_url)
        .await
        .expect("Failed to connect to test DB");

    let truncate_query = Statement::from_string(
        DbBackend::Postgres,
        "TRUNCATE TABLE samples, isolates, site_replicates, dna, sites RESTART IDENTITY CASCADE;"
            .to_owned(),
    );
    db.execute(truncate_query).await.unwrap();
    // Run migrations.
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

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
            "/api/site_replicates",
            sr_views::router(&db).split_for_parts().0,
        )
        .nest("/api/dna", dna_views::router(&db).split_for_parts().0)
        .nest("/api/isolates", iso_views::router(&db).split_for_parts().0)
        .nest("/api/samples", samp_views::router(&db).split_for_parts().0)
}
