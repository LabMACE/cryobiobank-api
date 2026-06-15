//! Smoke suite: asserts a best-case deployment is healthy and the Keycloak
//! write boundary behaves. Runs against the `test` compose profile (local
//! Keycloak + throwaway Postgres). All `#[ignore]` — the e2e watcher runs them
//! with `--include-ignored`; the Keycloak-gated ones self-skip when the local
//! Keycloak isn't up, so a plain `cargo test` stays green without the stack.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::test_utils::{
    build_app_with_db, build_app_with_keycloak, get_keycloak_jwt, keycloak_reachable,
    setup_clean_db,
};

async fn get(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

/// POST a JSON body, optionally with a bearer token, and return the status.
async fn post(app: &axum::Router, uri: &str, token: Option<&str>, body: Value) -> StatusCode {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json");
    if let Some(t) = token {
        builder = builder.header("Authorization", format!("Bearer {t}"));
    }
    let req = builder.body(Body::from(body.to_string())).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

#[tokio::test]
#[ignore]
async fn healthz_is_ok() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db);
    let (status, _) = get(&app, "/healthz").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn config_endpoint_serves_deployment_info() {
    let db = setup_clean_db().await;
    let app = build_app_with_db(db);
    let (status, body) = get(&app, "/api/config").await;
    assert_eq!(status, StatusCode::OK, "config: {body}");
    // The UI reads Keycloak settings from here on load.
    assert!(body.is_object(), "expected a config object, got {body}");
}

#[tokio::test]
#[ignore]
async fn migrated_db_serves_empty_list() {
    // setup_clean_db truncates + runs migrations; a fresh deployment should
    // answer list queries with an empty set (proves the schema is in place).
    let db = setup_clean_db().await;
    let app = build_app_with_db(db);
    let (status, body) = get(
        &app,
        "/api/sites?sort=%5B%22name%22%2C%22ASC%22%5D&range=%5B0%2C99%5D&filter=%7B%7D",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().map(|a| a.len()), Some(0));
}

#[tokio::test]
#[ignore]
async fn admin_jwt_can_write() {
    if !keycloak_reachable().await {
        eprintln!("skipping admin_jwt_can_write: local Keycloak unreachable");
        return;
    }
    let db = setup_clean_db().await;
    let app = build_app_with_keycloak(db).await;
    let token = get_keycloak_jwt("admin", "admin").await;
    let status = post(
        &app,
        "/api/areas/batch?partial=true",
        Some(&token),
        json!([{ "name": "Smoke Area", "colour": "#abcdef" }]),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "admin write should succeed");
}

#[tokio::test]
#[ignore]
async fn unauthenticated_write_is_rejected() {
    if !keycloak_reachable().await {
        eprintln!("skipping unauthenticated_write_is_rejected: local Keycloak unreachable");
        return;
    }
    let db = setup_clean_db().await;
    let app = build_app_with_keycloak(db).await;
    let status = post(
        &app,
        "/api/areas/batch?partial=true",
        None,
        json!([{ "name": "No Auth Area", "colour": "#000000" }]),
    )
    .await;
    // The Keycloak layer (Pass mode) always records an auth status, so a missing
    // token lands as a Failure → the write middleware returns 403, not 401.
    assert_eq!(status, StatusCode::FORBIDDEN, "no token → write rejected");
}

#[tokio::test]
#[ignore]
async fn non_admin_write_is_forbidden() {
    if !keycloak_reachable().await {
        eprintln!("skipping non_admin_write_is_forbidden: local Keycloak unreachable");
        return;
    }
    let db = setup_clean_db().await;
    let app = build_app_with_keycloak(db).await;
    let token = get_keycloak_jwt("user", "user").await;
    let status = post(
        &app,
        "/api/areas/batch?partial=true",
        Some(&token),
        json!([{ "name": "User Area", "colour": "#111111" }]),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "non-admin token → 403");
}
