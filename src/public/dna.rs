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

/// Public DNA model - excludes private field and private DNA records
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicDna {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub extraction_method: Option<String>,
    pub site_replicate_id: Uuid,
}

impl From<crate::dna::db::Model> for PublicDna {
    fn from(model: crate::dna::db::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            extraction_method: model.extraction_method,
            site_replicate_id: model.site_replicate_id,
        }
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db)
}

/// Get all public DNA records (non-private only, and only if all ancestors are not private)
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let mut condition = apply_filters(
        params.filter.clone(),
        &[
            ("name", crate::dna::db::Column::Name),
            ("extraction_method", crate::dna::db::Column::ExtractionMethod),
            ("site_replicate_id", crate::dna::db::Column::SiteReplicateId),
        ],
    );

    // Only show non-private DNA records
    condition = condition.add(crate::dna::db::Column::IsPrivate.eq(false));

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[("name", crate::dna::db::Column::Name)],
        crate::dna::db::Column::Name,
    );

    let public_replicate_ids = super::privacy::get_public_replicate_ids(&db).await;

    let objs = crate::dna::db::Entity::find()
        .filter(condition.clone())
        .filter(crate::dna::db::Column::SiteReplicateId.is_in(public_replicate_ids.clone()))
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    let response_objs: Vec<PublicDna> = objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = crate::dna::db::Entity::find()
        .filter(condition)
        .filter(crate::dna::db::Column::SiteReplicateId.is_in(public_replicate_ids))
        .count(&db)
        .await
        .unwrap();

    let mut headers = calculate_content_range(offset, limit, total_count, "dna");
    headers.insert("Access-Control-Expose-Headers", "Content-Range".parse().unwrap());

    (headers, Json(response_objs))
}

/// Get single public DNA record by ID (only if not private and all ancestors are not private)
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicDna>, (StatusCode, Json<String>)> {
    let dna = match crate::dna::db::Entity::find_by_id(id)
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

    // Check dna privacy
    if dna.is_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    // Check parent replicate/site/area privacy
    super::privacy::check_replicate_ancestry_public(&db, dna.site_replicate_id).await?;

    Ok(Json(PublicDna::from(dna)))
}