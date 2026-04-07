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

fn is_admin(req: &Request) -> bool {
    req.extensions()
        .get::<AuthStatus>()
        .map(|s| matches!(s, axum_keycloak_auth::KeycloakAuthStatus::Success(_)))
        .unwrap_or(false)
}

/// Block writes for non-admin, return early if unauthorized write attempt.
fn check_write_access(req: &Request) -> Option<Response> {
    if !is_admin(req) && *req.method() != Method::GET && *req.method() != Method::HEAD {
        Some(StatusCode::UNAUTHORIZED.into_response())
    } else {
        None
    }
}

/// Areas: `is_private = false`
pub async fn scope_areas(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(
            Condition::all().add(crate::areas::db::Column::IsPrivate.eq(false)),
        ));
    }
    next.run(req).await
}

/// Sites: `is_private = false AND (area_id IS NULL OR area not private)`
pub async fn scope_sites(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(
            Condition::all()
                .add(crate::sites::db::Column::IsPrivate.eq(false))
                .add(Expr::cust(
                    "(area_id IS NULL OR area_id NOT IN (SELECT id FROM areas WHERE is_private = true))",
                )),
        ));
    }
    next.run(req).await
}

/// Site replicates: `is_private = false AND site is public (with area check)`
pub async fn scope_site_replicates(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(
            Condition::all()
                .add(crate::sites::replicates::db::Column::IsPrivate.eq(false))
                .add(Expr::cust(
                    "site_id IN (\
                        SELECT s.id FROM sites s \
                        LEFT JOIN areas a ON s.area_id = a.id \
                        WHERE s.is_private = false \
                        AND (a.id IS NULL OR a.is_private = false)\
                    )",
                )),
        ));
    }
    next.run(req).await
}

const REPLICATE_SUBQUERY: &str = "\
    site_replicate_id IN (\
        SELECT sr.id FROM site_replicates sr \
        JOIN sites s ON sr.site_id = s.id \
        LEFT JOIN areas a ON s.area_id = a.id \
        WHERE sr.is_private = false AND s.is_private = false \
        AND (a.id IS NULL OR a.is_private = false)\
    )";

/// Samples: `is_private = false AND replicate/site/area chain is public`
pub async fn scope_samples(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(
            Condition::all()
                .add(crate::samples::db::Column::IsPrivate.eq(false))
                .add(Expr::cust(REPLICATE_SUBQUERY)),
        ));
    }
    next.run(req).await
}

/// Isolates: `is_private = false AND replicate/site/area chain is public`
pub async fn scope_isolates(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(
            Condition::all()
                .add(crate::isolates::db::Column::IsPrivate.eq(false))
                .add(Expr::cust(REPLICATE_SUBQUERY)),
        ));
    }
    next.run(req).await
}

/// DNA: `is_private = false AND replicate/site/area chain is public`
pub async fn scope_dna(mut req: Request, next: Next) -> Response {
    if let Some(r) = check_write_access(&req) { return r; }
    if !is_admin(&req) {
        req.extensions_mut().insert(ScopeCondition::new(
            Condition::all()
                .add(crate::dna::db::Column::IsPrivate.eq(false))
                .add(Expr::cust(REPLICATE_SUBQUERY)),
        ));
    }
    next.run(req).await
}
