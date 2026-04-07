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
use crate::common::enums::SampleType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Public isolate model - excludes private field and private isolates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicIsolate {
    pub id: Uuid,
    pub site_replicate_id: Uuid,
    pub name: String,
    pub taxonomy: Option<String>,
    pub temperature_of_isolation: Option<f64>,
    pub media_used_for_isolation: Option<String>,
    pub storage_location: Option<String>,
    pub genome_url: Option<String>,
    pub sample_type: SampleType,
}

impl From<crate::isolates::db::Model> for PublicIsolate {
    fn from(model: crate::isolates::db::Model) -> Self {
        Self {
            id: model.id,
            site_replicate_id: model.site_replicate_id,
            name: model.name,
            taxonomy: model.taxonomy,
            temperature_of_isolation: model.temperature_of_isolation,
            media_used_for_isolation: model.media_used_for_isolation,
            storage_location: model.storage_location,
            genome_url: model.genome_url,
            sample_type: model.sample_type,
        }
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db)
}

/// Get all public isolates (non-private only, and only if all ancestors are not private)
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {

    let (offset, limit) = parse_range(params.range.clone());

    let mut condition = apply_filters(
        params.filter.clone(),
        &[
            ("name", crate::isolates::db::Column::Name),
            ("taxonomy", crate::isolates::db::Column::Taxonomy),
            ("temperature_of_isolation", crate::isolates::db::Column::TemperatureOfIsolation),
            ("media_used_for_isolation", crate::isolates::db::Column::MediaUsedForIsolation),
            ("storage_location", crate::isolates::db::Column::StorageLocation),
            ("sample_type", crate::isolates::db::Column::SampleType),
            ("site_replicate_id", crate::isolates::db::Column::SiteReplicateId),
        ],
    );

    // Only show non-private isolates
    condition = condition.add(crate::isolates::db::Column::IsPrivate.eq(false));

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[
            ("name", crate::isolates::db::Column::Name),
            ("taxonomy", crate::isolates::db::Column::Taxonomy),
            ("sample_type", crate::isolates::db::Column::SampleType),
        ],
        crate::isolates::db::Column::Name,
    );

    let public_replicate_ids = super::privacy::get_public_replicate_ids(&db).await;

    // Run data fetch and count in parallel
    let (objs, total_count) = tokio::join!(
        crate::isolates::db::Entity::find()
            .filter(condition.clone())
            .filter(crate::isolates::db::Column::SiteReplicateId.is_in(public_replicate_ids.clone()))
            .order_by(order_column, order_direction)
            .offset(offset)
            .limit(limit)
            .all(&db),
        crate::isolates::db::Entity::find()
            .filter(condition)
            .filter(crate::isolates::db::Column::SiteReplicateId.is_in(public_replicate_ids))
            .count(&db),
    );
    let objs = objs.unwrap();
    let total_count: u64 = total_count.unwrap();

    let response_objs: Vec<PublicIsolate> = objs.into_iter().map(|obj| obj.into()).collect();

    let mut headers = calculate_content_range(offset, limit, total_count, "isolates");
    headers.insert("Access-Control-Expose-Headers", "Content-Range".parse().unwrap());

    (headers, Json(response_objs))
}

/// Get single public isolate by ID (only if not private and all ancestors are not private)
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicIsolate>, (StatusCode, Json<String>)> {
    let isolate = match crate::isolates::db::Entity::find_by_id(id)
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

    // Check isolate privacy
    if isolate.is_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    // Check parent replicate/site/area privacy
    super::privacy::check_replicate_ancestry_public(&db, isolate.site_replicate_id).await?;

    Ok(Json(PublicIsolate::from(isolate)))
}