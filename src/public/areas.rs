use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing, Json, Router,
};
use sea_orm::{
    query::*, ColumnTrait, DatabaseConnection, EntityTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Public area model - excludes private field and private areas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicArea {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
    pub geom: Option<serde_json::Value>,
}

impl PublicArea {
    /// Create PublicArea from database model with geometry data
    pub async fn from_model_with_geom(model: crate::areas::db::Model, db: &sea_orm::DatabaseConnection) -> Self {
        // Get geometry data using the same service as the original areas module
        let geom_data = crate::areas::services::get_convex_hull(db, model.id).await;

        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            colour: Some(model.colour),
            geom: geom_data,
        }
    }
}

impl From<crate::areas::db::Model> for PublicArea {
    fn from(model: crate::areas::db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            colour: Some(model.colour),
            geom: None, // Geometry not loaded in this conversion
        }
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db)
}

/// Get all public areas (non-private only)
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> Result<(axum::http::HeaderMap, Json<Vec<PublicArea>>), (StatusCode, String)> {
    let (offset, limit) = parse_range(params.range.clone());

    let mut condition = apply_filters(params.filter.clone(), &[("name", crate::areas::db::Column::Name)]);
    
    // Only show non-private areas
    condition = condition.add(crate::areas::db::Column::IsPrivate.eq(false));

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[("name", crate::areas::db::Column::Name)],
        crate::areas::db::Column::Name,
    );

    let objs = crate::areas::db::Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    // Process each area with geometry data (same as original areas module)
    let mut response_objs: Vec<PublicArea> = Vec::new();
    for obj in objs {
        let public_area = PublicArea::from_model_with_geom(obj, &db).await;
        response_objs.push(public_area);
    }

    let total_count: u64 = crate::areas::db::Entity::find()
        .filter(condition)
        .count(&db)
        .await
        .unwrap();

    let mut headers = calculate_content_range(offset, limit, total_count, "areas");
    headers.insert("Access-Control-Expose-Headers", "Content-Range".parse().unwrap());

    Ok((headers, Json(response_objs)))
}

/// Get single public area by ID (only if not private)
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicArea>, (StatusCode, Json<String>)> {
    let obj = match crate::areas::db::Entity::find_by_id(id)
        .filter(crate::areas::db::Column::IsPrivate.eq(false))
        .one(&db)
        .await
    {
        Ok(Some(obj)) => obj,
        Ok(None) => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            ))
        }
    };

    let public_area = PublicArea::from_model_with_geom(obj, &db).await;
    Ok(Json(public_area))
}