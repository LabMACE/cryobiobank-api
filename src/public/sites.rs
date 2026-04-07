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
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Public site model - includes aggregated sample types and replicate IDs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicSite {
    pub id: Uuid,
    pub name: String,
    pub longitude_4326: f64,
    pub latitude_4326: f64,
    pub elevation_metres: f64,
    pub area_id: Option<Uuid>,
    pub sample_types: Vec<String>,
    pub replicate_ids: Vec<Uuid>,
}

pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db)
}

/// Enrichment data for sites: sample types + replicate IDs
struct SiteEnrichment {
    sample_types: HashMap<Uuid, Vec<String>>,
    replicate_ids: HashMap<Uuid, Vec<Uuid>>,
}

/// Collect sample types and replicate IDs for a set of site IDs
/// by joining through site_replicates → samples AND isolates
async fn enrich_sites(db: &DatabaseConnection, site_ids: &[Uuid]) -> SiteEnrichment {
    if site_ids.is_empty() {
        return SiteEnrichment {
            sample_types: HashMap::new(),
            replicate_ids: HashMap::new(),
        };
    }

    // Fetch non-private replicates for these sites
    let replicates = crate::sites::replicates::db::Entity::find()
        .filter(crate::sites::replicates::db::Column::SiteId.is_in(site_ids.to_vec()))
        .filter(crate::sites::replicates::db::Column::IsPrivate.eq(false))
        .all(db)
        .await
        .unwrap_or_default();

    if replicates.is_empty() {
        return SiteEnrichment {
            sample_types: HashMap::new(),
            replicate_ids: HashMap::new(),
        };
    }

    // Build replicate_id → site_id lookup and site_id → replicate_ids
    let mut rep_to_site: HashMap<Uuid, Uuid> = HashMap::new();
    let mut site_rep_ids: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for r in &replicates {
        rep_to_site.insert(r.id, r.site_id);
        site_rep_ids.entry(r.site_id).or_default().push(r.id);
    }

    let rep_ids: Vec<Uuid> = replicates.iter().map(|r| r.id).collect();
    let mut site_types: HashMap<Uuid, HashSet<String>> = HashMap::new();

    // Fetch samples and isolates in parallel
    let (samples, isolates) = tokio::join!(
        crate::samples::db::Entity::find()
            .filter(crate::samples::db::Column::SiteReplicateId.is_in(rep_ids.clone()))
            .filter(crate::samples::db::Column::IsPrivate.eq(false))
            .all(db),
        crate::isolates::db::Entity::find()
            .filter(crate::isolates::db::Column::SiteReplicateId.is_in(rep_ids))
            .filter(crate::isolates::db::Column::IsPrivate.eq(false))
            .all(db),
    );
    let samples = samples.unwrap_or_default();
    let isolates = isolates.unwrap_or_default();

    for sample in &samples {
        if let Some(&site_id) = rep_to_site.get(&sample.site_replicate_id) {
            site_types
                .entry(site_id)
                .or_default()
                .insert(sample.sample_type.to_string());
        }
    }

    for isolate in &isolates {
        if let Some(&site_id) = rep_to_site.get(&isolate.site_replicate_id) {
            site_types
                .entry(site_id)
                .or_default()
                .insert(isolate.sample_type.to_string());
        }
    }

    let sample_types = site_types
        .into_iter()
        .map(|(k, v)| {
            let mut types: Vec<String> = v.into_iter().collect();
            types.sort();
            (k, types)
        })
        .collect();

    SiteEnrichment {
        sample_types,
        replicate_ids: site_rep_ids,
    }
}

fn build_public_site(
    obj: crate::sites::db::Model,
    enrichment: &SiteEnrichment,
) -> PublicSite {
    PublicSite {
        sample_types: enrichment
            .sample_types
            .get(&obj.id)
            .cloned()
            .unwrap_or_default(),
        replicate_ids: enrichment
            .replicate_ids
            .get(&obj.id)
            .cloned()
            .unwrap_or_default(),
        id: obj.id,
        name: obj.name,
        longitude_4326: obj.longitude_4326,
        latitude_4326: obj.latitude_4326,
        elevation_metres: obj.elevation_metres,
        area_id: obj.area_id,
    }
}

/// Get all public sites (non-private only, and only if parent area is not private)
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {

    let (offset, limit) = parse_range(params.range.clone());

    let mut condition =
        apply_filters(params.filter.clone(), &[("name", crate::sites::db::Column::Name)]);
    condition = condition.add(crate::sites::db::Column::IsPrivate.eq(false));

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[("name", crate::sites::db::Column::Name)],
        crate::sites::db::Column::Name,
    );


    let public_area_ids = crate::areas::db::Entity::find()
        .filter(crate::areas::db::Column::IsPrivate.eq(false))
        .select_only()
        .column(crate::areas::db::Column::Id)
        .into_tuple::<Uuid>()
        .all(&db)
        .await
        .unwrap_or_default();

    let objs = crate::sites::db::Entity::find()
        .filter(condition.clone())
        .filter(
            Condition::any()
                .add(crate::sites::db::Column::AreaId.is_null())
                .add(crate::sites::db::Column::AreaId.is_in(public_area_ids.clone()))
        )
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();


    let site_ids: Vec<Uuid> = objs.iter().map(|s| s.id).collect();
    let enrichment = enrich_sites(&db, &site_ids).await;


    let response_objs: Vec<PublicSite> = objs
        .into_iter()
        .map(|obj| build_public_site(obj, &enrichment))
        .collect();

    let total_count: u64 = crate::sites::db::Entity::find()
        .filter(condition)
        .filter(
            Condition::any()
                .add(crate::sites::db::Column::AreaId.is_null())
                .add(crate::sites::db::Column::AreaId.is_in(public_area_ids))
        )
        .count(&db)
        .await
        .unwrap();

    let mut headers = calculate_content_range(offset, limit, total_count, "sites");
    headers.insert(
        "Access-Control-Expose-Headers",
        "Content-Range".parse().unwrap(),
    );

    (headers, Json(response_objs))
}

/// Get single public site by ID (only if not private and parent area is not private)
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicSite>, (StatusCode, Json<String>)> {
    // Find the site first
    let site = match crate::sites::db::Entity::find_by_id(id).one(&db).await {
        Ok(Some(obj)) => obj,
        Ok(None) => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            ))
        }
    };

    // Check site privacy
    if site.is_private {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    // Check parent area privacy if area_id is set
    if let Some(area_id) = site.area_id {
        if let Ok(Some(area)) = crate::areas::db::Entity::find_by_id(area_id).one(&db).await {
            if area.is_private {
                return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
            }
        }
    }

    let enrichment = enrich_sites(&db, &[site.id]).await;
    Ok(Json(build_public_site(site, &enrichment)))
}
