use crate::config::Config;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbBackend, Statement};
use serde_json::Value;
use std::collections::HashMap;

/// Get convex hull for a single area (kept for backward compatibility with admin hooks)
pub async fn get_convex_hull(db: &DatabaseConnection, area_id: Uuid) -> Option<Value> {
    let hulls = get_convex_hulls_batch(db, &[area_id]).await;
    hulls.into_values().next()
}

/// Batch-fetch convex hulls for multiple areas in a single PostGIS query.
pub async fn get_convex_hulls_batch(
    db: &DatabaseConnection,
    area_ids: &[Uuid],
) -> HashMap<Uuid, Value> {
    if area_ids.is_empty() {
        return HashMap::new();
    }

    let config = Config::from_env();
    let raw_sql = r#"
    SELECT areas.id,
           ST_AsGeoJSON(ST_Transform(ST_Buffer(ST_Transform(ST_ConvexHull(ST_Collect(areas.geom)), 3857), $1), 4326)) AS convex_hull
    FROM (
        SELECT areas.id AS id,
               ST_SetSRID(ST_MakePoint(sites.longitude_4326, sites.latitude_4326), 4326) AS geom
        FROM areas
        JOIN sites ON areas.id = sites.area_id
    ) AS areas
    WHERE areas.id = ANY($2)
    GROUP BY areas.id
    "#;

    let rows = match db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            raw_sql,
            vec![
                config.area_buffer_metres.into(),
                area_ids.to_vec().into(),
            ],
        ))
        .await
    {
        Ok(rows) => rows,
        Err(_) => return HashMap::new(),
    };

    let mut result = HashMap::new();
    for row in rows {
        if let (Ok(id), Ok(hull_str)) = (
            row.try_get::<Uuid>("", "id"),
            row.try_get::<String>("", "convex_hull"),
        ) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&hull_str) {
                result.insert(id, parsed);
            }
        }
    }
    result
}
