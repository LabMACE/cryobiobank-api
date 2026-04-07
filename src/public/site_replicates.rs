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
use chrono::NaiveDate;
use sea_orm::{
    query::*, ColumnTrait, DatabaseConnection, EntityTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Public site replicate model - excludes private records and sensitive fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicSiteReplicate {
    pub id: Uuid,
    pub site_id: Uuid,
    pub name: String,
    pub sampling_date: NaiveDate,
    pub sample_depth_cm: Option<f64>,
    pub snow_depth_cm: Option<f64>,
    pub air_temperature_celsius: Option<f64>,
}

impl From<crate::sites::replicates::db::Model> for PublicSiteReplicate {
    fn from(model: crate::sites::replicates::db::Model) -> Self {
        Self {
            id: model.id,
            site_id: model.site_id,
            name: model.name,
            sampling_date: model.sampling_date,
            sample_depth_cm: model.sample_depth_cm,
            snow_depth_cm: model.snow_depth_cm,
            air_temperature_celsius: model.air_temperature_celsius,
        }
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db)
}

/// Get all public site replicates (non-private only, and only if parent site and its area are not private)
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {

    let (offset, limit) = parse_range(params.range.clone());

    let mut condition = apply_filters(
        params.filter.clone(),
        &[
            ("name", crate::sites::replicates::db::Column::Name),
            ("site_id", crate::sites::replicates::db::Column::SiteId),
            ("sampling_date", crate::sites::replicates::db::Column::SamplingDate),
        ],
    );

    // Only show non-private site replicates
    condition = condition.add(crate::sites::replicates::db::Column::IsPrivate.eq(false));

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[
            ("name", crate::sites::replicates::db::Column::Name),
            ("sampling_date", crate::sites::replicates::db::Column::SamplingDate),
        ],
        crate::sites::replicates::db::Column::Name,
    );

    let public_site_ids = super::privacy::get_public_site_ids(&db).await;

    let objs = crate::sites::replicates::db::Entity::find()
        .filter(condition.clone())
        .filter(crate::sites::replicates::db::Column::SiteId.is_in(public_site_ids.clone()))
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    let response_objs: Vec<PublicSiteReplicate> = objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = crate::sites::replicates::db::Entity::find()
        .filter(condition)
        .filter(crate::sites::replicates::db::Column::SiteId.is_in(public_site_ids))
        .count(&db)
        .await
        .unwrap();

    let mut headers = calculate_content_range(offset, limit, total_count, "site_replicates");
    headers.insert("Access-Control-Expose-Headers", "Content-Range".parse().unwrap());

    (headers, Json(response_objs))
}

/// Get single public site replicate by ID (only if not private and parent site/area are not private)
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicSiteReplicate>, (StatusCode, Json<String>)> {
    let replicate = match crate::sites::replicates::db::Entity::find_by_id(id)
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

    // Check replicate privacy
    if replicate.is_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    // Check parent site/area privacy
    super::privacy::check_site_ancestry_public(&db, replicate.site_id).await?;

    Ok(Json(PublicSiteReplicate::from(replicate)))
}
