mod areas;
mod common;
mod config;
mod dna;
mod isolates;
mod samples;
mod sites;

use axum::{routing::get, Router};
use axum_keycloak_auth::{instance::KeycloakAuthInstance, instance::KeycloakConfig, Url};
use config::Config;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[tokio::main]
async fn main() {
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

    let app: Router = Router::new()
        .route("/healthz", get(common::views::healthz))
        .route("/api/config", get(common::views::get_ui_config))
        .with_state(db.clone())
        .nest(
            "/api/sites",
            sites::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/site_replicates",
            sites::replicates::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/samples",
            samples::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/isolates",
            isolates::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/dna",
            dna::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
        )
        .nest(
            "/api/areas",
            areas::views::router(db.clone(), Some(keycloak_auth_instance.clone())),
            
        )
        // .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        // .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        ;

    let addr: std::net::SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("Listening on {}", addr);

    // Run the server
    let server = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app);

    // Wait for both the server and the background task to complete
    tokio::select! {
        res = server => {
            if let Err(err) = res {
                eprintln!("Server error: {}", err);
            }
        }
    }
}
