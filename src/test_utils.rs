use crate::{
    areas::db::Area as area_views, common::views as common_views, config::Config,
    dna::db::DNA as dna_views, field_records::db::FieldRecord as fr_views,
    isolates::db::Isolate as iso_views, middleware, samples::db::Sample as samp_views,
    sites::db::Site as sites_views,
};
use axum::{routing::get, Router};
use axum_keycloak_auth::{instance::KeycloakAuthInstance, instance::KeycloakConfig, Url};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema, Statement};
use std::sync::Arc;

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

// --- Keycloak-backed helpers (e2e stack) ------------------------------------
// These exercise the real auth boundary against the local Keycloak from the
// `test` compose profile. They read TEST_KEYCLOAK_* (where to mint JWTs) and,
// for the router, KEYCLOAK_URL/REALM (what the auth layer validates against) —
// the compose service sets both to the same local Keycloak.

fn test_keycloak_url() -> String {
    let raw = std::env::var("TEST_KEYCLOAK_URL").unwrap_or_else(|_| "http://localhost:8180/".into());
    if raw.ends_with('/') {
        raw
    } else {
        format!("{raw}/")
    }
}

fn test_keycloak_realm() -> String {
    std::env::var("TEST_KEYCLOAK_REALM").unwrap_or_else(|_| "cryobiobank-e2e".into())
}

fn test_keycloak_client_id() -> String {
    std::env::var("TEST_KEYCLOAK_CLIENT_ID").unwrap_or_else(|_| "cryobiobank-ui-local".into())
}

/// True when the local Keycloak's OIDC discovery is reachable. Keycloak-gated
/// tests self-skip when this is false, so a plain `cargo test` without the e2e
/// stack still runs the DB-only suites.
pub async fn keycloak_reachable() -> bool {
    let url = format!(
        "{}realms/{}/.well-known/openid-configuration",
        test_keycloak_url(),
        test_keycloak_realm()
    );
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Mint a real access token via the resource-owner password grant against the
/// local Keycloak (the public client has `directAccessGrantsEnabled`).
pub async fn get_keycloak_jwt(username: &str, password: &str) -> String {
    let url = format!(
        "{}realms/{}/protocol/openid-connect/token",
        test_keycloak_url(),
        test_keycloak_realm()
    );
    let client_id = test_keycloak_client_id();
    let resp = reqwest::Client::new()
        .post(&url)
        .form(&[
            ("grant_type", "password"),
            ("client_id", &client_id),
            ("username", username),
            ("password", password),
        ])
        .send()
        .await
        .expect("Keycloak unreachable");
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.expect("Keycloak returned non-JSON");
    assert!(status.is_success(), "Keycloak token request failed: {body}");
    body["access_token"]
        .as_str()
        .expect("no access_token in Keycloak response")
        .to_string()
}

/// Build the app WITH the real Keycloak auth layer, mirroring `main.rs`, so
/// bearer JWTs are validated against the local Keycloak's JWKS — the only path
/// that exercises the admin-only write boundary (`build_app_with_db` strips it).
/// Awaits OIDC discovery before returning so the first request can't race it.
pub async fn build_app_with_keycloak(db: DatabaseConnection) -> Router {
    let config = Config::from_env();

    let instance = Arc::new(KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse(&config.keycloak_url).unwrap())
            .realm(String::from(&config.keycloak_realm))
            .build(),
    ));

    // Bounded poll so a misconfigured Keycloak can't hang the test.
    for _ in 0..200 {
        if instance.is_operational().await {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let keycloak_pass_layer =
        axum_keycloak_auth::layer::KeycloakAuthLayer::<crate::common::auth::Role>::builder()
            .instance(instance)
            .passthrough_mode(axum_keycloak_auth::PassthroughMode::Pass)
            .persist_raw_claims(false)
            .expected_audiences(vec![String::from("account")])
            .required_roles(vec![crate::common::auth::Role::Administrator])
            .build();

    Router::new()
        .route("/healthz", get(common_views::healthz))
        .route("/api/config", get(common_views::get_ui_config))
        .with_state(db.clone())
        .nest(
            "/api/sites",
            Router::from(sites_views::router(&db))
                .layer(axum::middleware::from_fn(middleware::scope_sites)),
        )
        .nest(
            "/api/field_records",
            Router::from(fr_views::router(&db))
                .layer(axum::middleware::from_fn(middleware::scope_field_records)),
        )
        .nest(
            "/api/samples",
            Router::from(samp_views::router(&db))
                .layer(axum::middleware::from_fn(middleware::scope_samples)),
        )
        .nest(
            "/api/isolates",
            Router::from(iso_views::router(&db))
                .layer(axum::middleware::from_fn(middleware::scope_isolates)),
        )
        .nest(
            "/api/dna",
            Router::from(dna_views::router(&db))
                .layer(axum::middleware::from_fn(middleware::scope_dna)),
        )
        .nest(
            "/api/areas",
            Router::from(area_views::router(&db))
                .layer(axum::middleware::from_fn(middleware::scope_areas)),
        )
        .route(
            "/api/search",
            get(crate::search::search).with_state(db.clone()),
        )
        .layer(keycloak_pass_layer)
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
