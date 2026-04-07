use axum::http::StatusCode;
use axum::Json;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

type PublicError = (StatusCode, Json<String>);

/// Get IDs of all public site replicates (non-private, with public parent site and area).
/// Single SQL query with JOINs instead of 3 separate queries.
pub async fn get_public_replicate_ids(db: &DatabaseConnection) -> Vec<Uuid> {
    let rows = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            r"SELECT sr.id
              FROM site_replicates sr
              JOIN sites s ON sr.site_id = s.id
              LEFT JOIN areas a ON s.area_id = a.id
              WHERE sr.is_private = false
                AND s.is_private = false
                AND (a.id IS NULL OR a.is_private = false)"
                .to_owned(),
        ))
        .await
        .unwrap_or_default();

    rows.iter()
        .filter_map(|row| row.try_get::<Uuid>("", "id").ok())
        .collect()
}

/// Get IDs of all public sites (non-private, with public parent area).
/// Single SQL query instead of 2 separate queries.
pub async fn get_public_site_ids(db: &DatabaseConnection) -> Vec<Uuid> {
    let rows = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            r"SELECT s.id
              FROM sites s
              LEFT JOIN areas a ON s.area_id = a.id
              WHERE s.is_private = false
                AND (a.id IS NULL OR a.is_private = false)"
                .to_owned(),
        ))
        .await
        .unwrap_or_default();

    rows.iter()
        .filter_map(|row| row.try_get::<Uuid>("", "id").ok())
        .collect()
}

/// Check that a site and its parent area are public.
pub async fn check_site_ancestry_public(
    db: &DatabaseConnection,
    site_id: Uuid,
) -> Result<(), PublicError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r"SELECT s.is_private AS site_private,
                     a.is_private AS area_private
              FROM sites s
              LEFT JOIN areas a ON s.area_id = a.id
              WHERE s.id = $1",
            vec![site_id.into()],
        ))
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error".to_string())))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json("Not Found".to_string())))?;

    let site_private: bool = row.try_get("", "site_private")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error".to_string())))?;
    if site_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    let area_private: Option<bool> = row.try_get("", "area_private").ok();
    if area_private == Some(true) {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    Ok(())
}

/// Check that a site replicate and all its ancestors (site, area) are public.
/// Single SQL query with JOINs instead of 3 sequential queries.
pub async fn check_replicate_ancestry_public(
    db: &DatabaseConnection,
    replicate_id: Uuid,
) -> Result<(), PublicError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r"SELECT sr.is_private AS rep_private,
                     s.is_private AS site_private,
                     a.is_private AS area_private
              FROM site_replicates sr
              JOIN sites s ON sr.site_id = s.id
              LEFT JOIN areas a ON s.area_id = a.id
              WHERE sr.id = $1",
            vec![replicate_id.into()],
        ))
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error".to_string())))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json("Not Found".to_string())))?;

    let rep_private: bool = row.try_get("", "rep_private")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error".to_string())))?;
    if rep_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    let site_private: bool = row.try_get("", "site_private")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal Server Error".to_string())))?;
    if site_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    let area_private: Option<bool> = row.try_get("", "area_private").ok();
    if area_private == Some(true) {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    Ok(())
}
