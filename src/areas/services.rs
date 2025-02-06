use crate::config::Config;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbBackend, Statement};
use serde_json::Value;

pub async fn get_convex_hull(db: &DatabaseConnection, area_id: Uuid) -> Option<Value> {
    let config: Config = Config::from_env();
    let raw_sql = r#"
    SELECT areas.id,
           ST_AsGeoJSON(ST_Transform(ST_Buffer(ST_Transform(ST_ConvexHull(ST_Collect(areas.geom)), 3857), $2), 4326)) AS convex_hull
    FROM (
        SELECT areas.id AS id,
               ST_SetSRID(ST_MakePoint(sites.longitude_4326, sites.latitude_4326), 4326) AS geom
        FROM areas
        JOIN sites ON areas.id = sites.area_id
    ) AS areas
    WHERE areas.id = $1
    GROUP BY areas.id
    "#;

    // Try to execute the query
    if let Ok(result) = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            raw_sql,
            vec![
                area_id.into(),                   // The ID of the area
                config.area_buffer_metres.into(), // The buffer distance in metres
            ],
        ))
        .await
    {
        if let Some(row) = result {
            if let Ok(convex_hull) = row.try_get::<String>("", "convex_hull") {
                if let Ok(parsed_geojson) = serde_json::from_str(&convex_hull) {
                    return parsed_geojson; // Return the parsed GeoJSON if valid
                }
            }
        }
    }

    // Return none if the query fails
    None
}
