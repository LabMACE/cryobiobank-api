use crate::common::auth::Role;
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
use axum_keycloak_auth::{
    instance::KeycloakAuthInstance, layer::KeycloakAuthLayer, PassthroughMode,
};
use sea_orm::{query::*, DatabaseConnection, EntityTrait};
use std::sync::Arc;
use uuid::Uuid;

pub fn router(db: DatabaseConnection, keycloak_auth_instance: Arc<KeycloakAuthInstance>) -> Router {
    Router::new()
        .route(
            "/",
            routing::get(get_all), // .post(create_one)
        )
        .route(
            "/:id",
            routing::get(get_one), // .put(update_one).delete(delete_one)
        )
        // .route(
        //     "/:id",
        //     routing::get(get_one).put(update_one).delete(delete_one),
        // )
        .with_state(db)
        .layer(
            KeycloakAuthLayer::<Role>::builder()
                .instance(keycloak_auth_instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![Role::Administrator])
                .build(),
        )
}

const RESOURCE_NAME: &str = "isolates";

#[utoipa::path(
    get,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = OK, body = super::models::DNA))
)]
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let condition = apply_filters(
        params.filter.clone(),
        &[
            ("name", super::db::Column::Name),
            ("taxonomy", super::db::Column::Taxonomy),
            (
                "media_used_for_isolation",
                super::db::Column::MediaUsedForIsolation,
            ),
            ("storage_location", super::db::Column::StorageLocation),
        ],
    );

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[
            ("id", super::db::Column::Id),
            ("site_replicate_id", super::db::Column::SiteReplicateId),
            ("dna_id", super::db::Column::DnaId),
            ("name", super::db::Column::Name),
            ("taxonomy", super::db::Column::Taxonomy),
            (
                "temperature_of_isolation",
                super::db::Column::TemperatureOfIsolation,
            ),
            (
                "media_used_for_isolation",
                super::db::Column::MediaUsedForIsolation,
            ),
            ("storage_location", super::db::Column::StorageLocation),
        ],
        super::db::Column::Id,
    );

    let objs: Vec<super::db::Model> = super::db::Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    // Map the results from the database models
    let response_objs: Vec<super::models::Isolate> =
        objs.into_iter().map(|obj| obj.into()).collect();

    let total_count: u64 = <super::db::Entity>::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);

    (headers, Json(response_objs))
}
#[utoipa::path(
    get,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Submission))
)]
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<super::models::Isolate>, (StatusCode, Json<String>)> {
    let obj = match super::db::Entity::find_by_id(id).one(&db).await {
        Ok(obj) => obj.unwrap(),
        _ => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
    };

    let submission: super::models::Isolate = obj.clone().into();

    Ok(Json(submission))
}
