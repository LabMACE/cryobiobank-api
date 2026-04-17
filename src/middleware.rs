use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use crudcrate::ScopeCondition;
use sea_orm::{sea_query::Expr, ColumnTrait, Condition};

use crate::common::auth::Role;

type AuthStatus = axum_keycloak_auth::KeycloakAuthStatus<Role, axum_keycloak_auth::decode::ProfileAndEmail>;

pub fn is_admin(req: &Request) -> bool {
    req.extensions()
        .get::<AuthStatus>()
        .map(|s| matches!(s, axum_keycloak_auth::KeycloakAuthStatus::Success(_)))
        .unwrap_or(false)
}

/// Block writes for non-admin, return early if unauthorized/forbidden write attempt.
fn check_write_access(req: &Request) -> Option<Response> {
    if *req.method() != Method::GET && *req.method() != Method::HEAD && !is_admin(req) {
        match req.extensions().get::<AuthStatus>() {
            Some(axum_keycloak_auth::KeycloakAuthStatus::Failure(_)) => {
                Some(StatusCode::FORBIDDEN.into_response())
            }
            None => Some(StatusCode::UNAUTHORIZED.into_response()),
            _ => Some(StatusCode::FORBIDDEN.into_response()),
        }
    } else {
        None
    }
}

const REPLICATE_SUBQUERY: &str = "\
    site_replicate_id IN (\
        SELECT sr.id FROM site_replicates sr \
        JOIN sites s ON sr.site_id = s.id \
        LEFT JOIN areas a ON s.area_id = a.id \
        WHERE sr.is_private = false AND s.is_private = false \
        AND (a.id IS NULL OR a.is_private = false)\
    )";

pub fn areas_scope() -> Condition {
    Condition::all().add(crate::areas::db::Column::IsPrivate.eq(false))
}

pub fn sites_scope() -> Condition {
    Condition::all()
        .add(crate::sites::db::Column::IsPrivate.eq(false))
        .add(Expr::cust(
            "(area_id IS NULL OR area_id NOT IN (SELECT id FROM areas WHERE is_private = true))",
        ))
}

pub fn site_replicates_scope() -> Condition {
    Condition::all()
        .add(crate::sites::replicates::db::Column::IsPrivate.eq(false))
        .add(Expr::cust(
            "site_id IN (\
                SELECT s.id FROM sites s \
                LEFT JOIN areas a ON s.area_id = a.id \
                WHERE s.is_private = false \
                AND (a.id IS NULL OR a.is_private = false)\
            )",
        ))
}

pub fn samples_scope() -> Condition {
    Condition::all()
        .add(crate::samples::db::Column::IsPrivate.eq(false))
        .add(Expr::cust(REPLICATE_SUBQUERY))
}

pub fn isolates_scope() -> Condition {
    Condition::all()
        .add(crate::isolates::db::Column::IsPrivate.eq(false))
        .add(Expr::cust(REPLICATE_SUBQUERY))
}

pub fn dna_scope() -> Condition {
    Condition::all()
        .add(crate::dna::db::Column::IsPrivate.eq(false))
        .add(Expr::cust(REPLICATE_SUBQUERY))
}

/// Areas: `is_private = false`
pub async fn scope_areas(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(areas_scope()));
    }
    next.run(req).await
}

/// Sites: `is_private = false AND (area_id IS NULL OR area not private)`
pub async fn scope_sites(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(sites_scope()));
    }
    next.run(req).await
}

/// Site replicates: `is_private = false AND site is public (with area check)`
pub async fn scope_site_replicates(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(site_replicates_scope()));
    }
    next.run(req).await
}

/// Samples: `is_private = false AND replicate/site/area chain is public`
pub async fn scope_samples(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(samples_scope()));
    }
    next.run(req).await
}

/// Isolates: `is_private = false AND replicate/site/area chain is public`
pub async fn scope_isolates(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(isolates_scope()));
    }
    next.run(req).await
}

/// DNA: `is_private = false AND replicate/site/area chain is public`
pub async fn scope_dna(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(dna_scope()));
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::test_utils::{build_app_with_db, build_scoped_app_with_db, setup_clean_db};

    async fn admin_create(app: &axum::Router, path: &str, payload: Value) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        (status, json)
    }

    async fn scoped_get(app: &axum::Router, path: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        (status, json)
    }

    #[tokio::test]
    #[ignore]
    async fn scoped_list_excludes_private_areas() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let (s, _) = admin_create(&admin, "/api/areas", json!({
            "name": "Public Area", "description": "visible", "colour": "#00ff00", "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);
        let (s, _) = admin_create(&admin, "/api/areas", json!({
            "name": "Private Area", "description": "hidden", "colour": "#ff0000", "is_private": true
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        let (status, body) = scoped_get(&scoped, "/api/areas").await;
        assert_eq!(status, StatusCode::OK);
        let areas = body.as_array().expect("response should be an array");
        assert_eq!(areas.len(), 1, "Only public area should be visible");
        assert_eq!(areas[0]["name"], "Public Area");
    }

    #[tokio::test]
    #[ignore]
    async fn scoped_get_one_returns_404_for_private() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let (_, created) = admin_create(&admin, "/api/areas", json!({
            "name": "Secret Area", "description": "hidden", "colour": "#ff0000", "is_private": true
        })).await;
        let id = created["id"].as_str().unwrap();
        let (status, _) = scoped_get(&scoped, &format!("/api/areas/{id}")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn scoped_site_hidden_when_parent_area_private() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let (_, area) = admin_create(&admin, "/api/areas", json!({
            "name": "Hidden Area", "description": "private", "colour": "#ff0000", "is_private": true
        })).await;
        let area_id = area["id"].as_str().unwrap();

        let (s, _) = admin_create(&admin, "/api/sites", json!({
            "name": "Site Under Private Area", "latitude_4326": 46.0, "longitude_4326": 7.0,
            "elevation_metres": 1000.0, "area_id": area_id, "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);
        let (s, _) = admin_create(&admin, "/api/sites", json!({
            "name": "Standalone Public Site", "latitude_4326": 47.0, "longitude_4326": 8.0,
            "elevation_metres": 500.0, "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        let (status, body) = scoped_get(&scoped, "/api/sites").await;
        assert_eq!(status, StatusCode::OK);
        let sites = body.as_array().expect("response should be an array");
        assert_eq!(sites.len(), 1, "Only standalone public site should be visible");
        assert_eq!(sites[0]["name"], "Standalone Public Site");
    }

    #[tokio::test]
    #[ignore]
    async fn scoped_response_excludes_is_private_field() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let (_, created) = admin_create(&admin, "/api/areas", json!({
            "name": "Visible Area", "description": "public", "colour": "#00ff00", "is_private": false
        })).await;
        let id = created["id"].as_str().unwrap();

        let (_, body) = scoped_get(&scoped, "/api/areas").await;
        let areas = body.as_array().unwrap();
        assert!(!areas.is_empty());
        assert!(areas[0].get("is_private").is_none(), "is_private should not be in scoped list response");

        let (status, item) = scoped_get(&scoped, &format!("/api/areas/{id}")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(item.get("is_private").is_none(), "is_private should not be in scoped get_one response");
    }

    #[tokio::test]
    #[ignore]
    async fn scoped_write_returns_forbidden() {
        let db = setup_clean_db().await;
        let scoped = build_scoped_app_with_db(db.clone());

        let request = Request::builder()
            .method("POST").uri("/api/areas")
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"name": "hack", "description": "", "colour": "#000", "is_private": false}).to_string()))
            .unwrap();
        let response = scoped.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let request = Request::builder()
            .method("DELETE").uri("/api/areas/00000000-0000-0000-0000-000000000001")
            .body(Body::empty()).unwrap();
        let response = scoped.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let request = Request::builder()
            .method("PUT").uri("/api/areas/00000000-0000-0000-0000-000000000001")
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"name": "hacked"}).to_string())).unwrap();
        let response = scoped.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[ignore]
    async fn scoped_full_hierarchy_privacy_chain() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let (_, area) = admin_create(&admin, "/api/areas", json!({
            "name": "Private Root", "description": "root", "colour": "#ff0000", "is_private": true
        })).await;
        let area_id = area["id"].as_str().unwrap();

        let (_, site) = admin_create(&admin, "/api/sites", json!({
            "name": "Child Site", "latitude_4326": 46.0, "longitude_4326": 7.0,
            "elevation_metres": 1000.0, "area_id": area_id, "is_private": false
        })).await;
        let site_id = site["id"].as_str().unwrap();

        let (_, replicate) = admin_create(&admin, "/api/site_replicates", json!({
            "name": "Replicate 1", "site_id": site_id,
            "sample_type": "Snow",
            "sampling_date": "2024-06-01",
            "is_private": false
        })).await;
        let replicate_id = replicate["id"].as_str().unwrap();

        let (s, _) = admin_create(&admin, "/api/samples", json!({
            "name": "Sample 1", "site_replicate_id": replicate_id, "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        let (s, _) = admin_create(&admin, "/api/isolates", json!({
            "name": "Isolate 1", "site_replicate_id": replicate_id, "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        let (s, _) = admin_create(&admin, "/api/dna", json!({
            "name": "DNA 1", "site_replicate_id": replicate_id, "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        for endpoint in ["/api/sites", "/api/site_replicates", "/api/samples", "/api/isolates", "/api/dna"] {
            let (status, body) = scoped_get(&scoped, endpoint).await;
            assert_eq!(status, StatusCode::OK, "GET {endpoint} should succeed");
            let items = body.as_array().expect("response should be an array");
            assert!(items.is_empty(), "GET {endpoint} should return empty list when root area is private, got {} items", items.len());
        }
    }

    /// Seeds a public site + public replicate with one public and one private
    /// row in each of samples/isolates/dna and returns the replicate id.
    ///
    /// The public site_replicate is the parent for all children. We avoid the
    /// site/area chain here because we want the scope to succeed at the
    /// replicate level, forcing the scoped batch loader to produce rows — if
    /// the chain filters the replicate out, the whole join is empty and the
    /// test proves nothing.
    async fn seed_replicate_with_mixed_children(app: &axum::Router) -> String {
        let (s, site) = admin_create(app, "/api/sites", json!({
            "name": "PubSite",
            "latitude_4326": 46.0, "longitude_4326": 7.0, "elevation_metres": 1000.0,
            "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);
        let site_id = site["id"].as_str().unwrap();

        let (s, replicate) = admin_create(app, "/api/site_replicates", json!({
            "name": "PubReplicate", "site_id": site_id,
            "sample_type": "Snow",
            "sampling_date": "2024-06-01",
            "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);
        let replicate_id = replicate["id"].as_str().unwrap().to_string();

        for (is_private, suffix) in [(false, "pub"), (true, "priv")] {
            let (s, _) = admin_create(app, "/api/samples", json!({
                "name": format!("sample-{suffix}"),
                "site_replicate_id": replicate_id,
                "is_private": is_private
            })).await;
            assert_eq!(s, StatusCode::CREATED);

            let (s, _) = admin_create(app, "/api/isolates", json!({
                "name": format!("isolate-{suffix}"),
                "site_replicate_id": replicate_id,
                "is_private": is_private
            })).await;
            assert_eq!(s, StatusCode::CREATED);

            let (s, _) = admin_create(app, "/api/dna", json!({
                "name": format!("dna-{suffix}"),
                "site_replicate_id": replicate_id,
                "is_private": is_private
            })).await;
            assert_eq!(s, StatusCode::CREATED);
        }

        replicate_id
    }

    /// Public `GET /api/site_replicates` (list): embedded samples/isolates/dna
    /// must only contain public rows. Proves `get_all_scoped` propagates the
    /// child scope filter through crudcrate's batch loader on the list path.
    #[tokio::test]
    #[ignore]
    async fn scoped_replicate_list_excludes_private_joined_children() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        seed_replicate_with_mixed_children(&admin).await;

        let (status, body) = scoped_get(&scoped, "/api/site_replicates").await;
        assert_eq!(status, StatusCode::OK);
        let replicates = body.as_array().expect("response should be an array");
        assert_eq!(replicates.len(), 1, "exactly one public replicate expected");
        let r = &replicates[0];

        for field in ["samples", "isolates", "dna"] {
            let children = r[field].as_array()
                .unwrap_or_else(|| panic!("expected {field} array, got {r}"));
            assert_eq!(
                children.len(), 1,
                "{field}: expected only the public row in scoped list, got {} items: {children:?}",
                children.len()
            );
            let name = children[0]["name"].as_str().unwrap();
            assert!(
                name.ends_with("-pub"),
                "{field}: private child leaked into scoped list response (name={name})"
            );
        }
    }

    /// Public `GET /api/site_replicates/:id` (get_one): embedded children
    /// must only contain public rows. Proves `get_one_scoped`'s SQL-level
    /// child scope filter works for the Vec join at depth = 1.
    #[tokio::test]
    #[ignore]
    async fn scoped_replicate_one_excludes_private_joined_children() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let replicate_id = seed_replicate_with_mixed_children(&admin).await;

        let (status, r) = scoped_get(&scoped, &format!("/api/site_replicates/{replicate_id}")).await;
        assert_eq!(status, StatusCode::OK);
        for field in ["samples", "isolates", "dna"] {
            let children = r[field].as_array()
                .unwrap_or_else(|| panic!("expected {field} array, got {r}"));
            assert_eq!(
                children.len(), 1,
                "{field}: expected only the public row in scoped get_one, got {} items",
                children.len()
            );
            assert!(
                children[0]["name"].as_str().unwrap().ends_with("-pub"),
                "{field}: private child leaked into scoped get_one response"
            );
        }
    }

    /// Public `GET /api/areas/:id` (get_one at depth = 2): the sites array
    /// must not contain any private sites. Proves that the depth > 1 branch
    /// of the scoped join loader recurses via `get_one_scoped`, not `get_one`.
    #[tokio::test]
    #[ignore]
    async fn scoped_area_one_excludes_private_grandchildren() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        let (_, area) = admin_create(&admin, "/api/areas", json!({
            "name": "PubArea", "description": "public", "colour": "#00ff00", "is_private": false
        })).await;
        let area_id = area["id"].as_str().unwrap();

        let (s, _) = admin_create(&admin, "/api/sites", json!({
            "name": "PubSiteUnderArea",
            "latitude_4326": 46.0, "longitude_4326": 7.0, "elevation_metres": 1000.0,
            "area_id": area_id, "is_private": false
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        let (s, _) = admin_create(&admin, "/api/sites", json!({
            "name": "PrivSiteUnderArea",
            "latitude_4326": 47.0, "longitude_4326": 8.0, "elevation_metres": 500.0,
            "area_id": area_id, "is_private": true
        })).await;
        assert_eq!(s, StatusCode::CREATED);

        let (status, area_json) = scoped_get(&scoped, &format!("/api/areas/{area_id}")).await;
        assert_eq!(status, StatusCode::OK);
        let sites = area_json["sites"].as_array().expect("sites array");
        assert_eq!(
            sites.len(), 1,
            "expected only the public site via depth>1 scoped join, got {} entries: {sites:?}",
            sites.len()
        );
        assert_eq!(sites[0]["name"], "PubSiteUnderArea");
    }

    /// Scoped response shape guard: `is_private` must be absent from every
    /// nested child in the list response (belt-and-suspenders on top of
    /// exclude(scoped)).
    #[tokio::test]
    #[ignore]
    async fn scoped_replicate_list_is_private_absent_on_children() {
        let db = setup_clean_db().await;
        let admin = build_app_with_db(db.clone());
        let scoped = build_scoped_app_with_db(db.clone());

        seed_replicate_with_mixed_children(&admin).await;

        let (status, body) = scoped_get(&scoped, "/api/site_replicates").await;
        assert_eq!(status, StatusCode::OK);
        let replicates = body.as_array().unwrap();
        assert_eq!(replicates.len(), 1);
        let r = &replicates[0];
        assert!(
            r.get("is_private").is_none(),
            "replicate itself should not expose is_private in scoped response"
        );
        for field in ["samples", "isolates", "dna"] {
            for child in r[field].as_array().unwrap() {
                assert!(
                    child.get("is_private").is_none(),
                    "{field} child leaked is_private field: {child}"
                );
            }
        }
    }
}
