mod areas;
mod common;
mod config;
mod dna;
mod isolates;
mod middleware;
mod samples;
mod sites;
#[cfg(test)]
mod test_utils;

use axum::{routing::get, Router};
use axum_keycloak_auth::{instance::KeycloakAuthInstance, instance::KeycloakConfig, Url};
use config::Config;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap()
        .block_on(run())
}

async fn run() {
    let config = Config::from_env();

    let db: DatabaseConnection = Database::connect(&*config.db_url.as_ref().unwrap())
        .await
        .unwrap();

    if db.ping().await.is_ok() {
        println!("Connected to the database");
    } else {
        println!("Could not connect to the database");
    }

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    println!(
        "Starting server {} ({} deployment) ...",
        config.app_name,
        config.deployment.to_uppercase()
    );

    let keycloak_auth_instance: Arc<KeycloakAuthInstance> = Arc::new(KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse(&config.keycloak_url).unwrap())
            .realm(String::from(&config.keycloak_realm))
            .build(),
    ));

    // Keycloak layer in Pass mode — lets all requests through, stores auth status in extensions
    let keycloak_pass_layer =
        axum_keycloak_auth::layer::KeycloakAuthLayer::<common::auth::Role>::builder()
            .instance(keycloak_auth_instance.clone())
            .passthrough_mode(axum_keycloak_auth::PassthroughMode::Pass)
            .persist_raw_claims(false)
            .expected_audiences(vec![String::from("account")])
            .required_roles(vec![common::auth::Role::Administrator])
            .build();

    // Single API surface: auth state determines behavior
    // - Admin (authenticated): full CRUD, no filtering
    // - Public (unauthenticated): read-only, privacy-filtered via ScopeCondition
    let app: Router = Router::new()
        .route("/healthz", get(common::views::healthz))
        .route("/api/config", get(common::views::get_ui_config))
        .with_state(db.clone())
        .nest(
            "/api/sites",
            Router::from(sites::db::Site::router(&db.clone()))
                .layer(axum::middleware::from_fn(middleware::scope_sites)),
        )
        .nest(
            "/api/site_replicates",
            Router::from(sites::replicates::db::SiteReplicate::router(&db.clone()))
                .layer(axum::middleware::from_fn(middleware::scope_site_replicates)),
        )
        .nest(
            "/api/samples",
            Router::from(samples::db::Sample::router(&db.clone()))
                .layer(axum::middleware::from_fn(middleware::scope_samples)),
        )
        .nest(
            "/api/isolates",
            Router::from(isolates::db::Isolate::router(&db.clone()))
                .layer(axum::middleware::from_fn(middleware::scope_isolates)),
        )
        .nest(
            "/api/dna",
            Router::from(dna::db::DNA::router(&db.clone()))
                .layer(axum::middleware::from_fn(middleware::scope_dna)),
        )
        .nest(
            "/api/areas",
            Router::from(areas::db::Area::router(&db.clone()))
                .layer(axum::middleware::from_fn(middleware::scope_areas)),
        )
        .layer(keycloak_pass_layer);

    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {}", addr);

    let server = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app);

    tokio::select! {
        res = server => {
            if let Err(err) = res {
                eprintln!("Server error: {}", err);
            }
        }
    }
}
