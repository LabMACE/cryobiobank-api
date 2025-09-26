use crate::common::filter::{apply_filters, parse_range};
use crate::common::models::FilterOptions;
use crate::common::pagination::calculate_content_range;
use crate::common::sort::generic_sort;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use sea_orm::{
    query::*, ColumnTrait, DatabaseConnection, EntityTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Public sample model - excludes private field and private samples
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicSample {
    pub id: Uuid,
    pub site_replicate_id: Uuid,
    pub name: String,
    pub type_of_sample: Option<String>,
    pub storage_location: Option<String>,
    pub description: Option<String>,
    pub dna_id: Option<Uuid>,
}

impl From<crate::samples::db::Model> for PublicSample {
    fn from(model: crate::samples::db::Model) -> Self {
        Self {
            id: model.id,
            site_replicate_id: model.site_replicate_id,
            name: model.name,
            type_of_sample: model.type_of_sample,
            storage_location: model.storage_location,
            description: model.description,
            dna_id: model.dna_id,
        }
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db)
}

/// Get all public samples (non-private only)
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let mut condition = apply_filters(
        params.filter.clone(),
        &[
            ("name", crate::samples::db::Column::Name),
            ("type_of_sample", crate::samples::db::Column::TypeOfSample),
            ("storage_location", crate::samples::db::Column::StorageLocation),
        ],
    );
    
    // Only show non-private samples
    condition = condition.add(crate::samples::db::Column::IsPrivate.eq(false));

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[("name", crate::samples::db::Column::Name)],
        crate::samples::db::Column::Name,
    );

    let objs = crate::samples::db::Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    let response_objs: Vec<PublicSample> = objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = crate::samples::db::Entity::find()
        .filter(condition)
        .count(&db)
        .await
        .unwrap();

    let mut headers = calculate_content_range(offset, limit, total_count, "samples");
    headers.insert("Access-Control-Expose-Headers", "Content-Range".parse().unwrap());

    (headers, Json(response_objs))
}

/// Get single public sample by ID (only if not private)
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicSample>, (StatusCode, Json<String>)> {
    let obj = match crate::samples::db::Entity::find_by_id(id)
        .filter(crate::samples::db::Column::IsPrivate.eq(false))
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

    Ok(Json(PublicSample::from(obj)))
}