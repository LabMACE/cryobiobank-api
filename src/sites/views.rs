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
use sea_orm::{
    query::*, DatabaseConnection, EntityTrait, IntoActiveModel, LoaderTrait, ModelTrait, SqlErr,
};
use sea_orm::{ActiveModelTrait, DeleteResult};
use std::sync::Arc;
use uuid::Uuid;

pub fn router(
    db: DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> Router {
    let mut router = Router::new()
        .route("/", routing::get(get_all))
        .route(
            "/{id}",
            routing::get(get_one).put(update_one).delete(delete_one),
        )
        .with_state(db);

    if let Some(instance) = keycloak_auth_instance {
        router = router.layer(
            KeycloakAuthLayer::<Role>::builder()
                .instance(instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![Role::Administrator])
                .build(),
        );
    }

    router
}

const RESOURCE_NAME: &str = "sites";

#[utoipa::path(
    get,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Site))
)]
pub async fn get_all(
    Query(params): Query<FilterOptions>,
    State(db): State<DatabaseConnection>,
) -> impl IntoResponse {
    let (offset, limit) = parse_range(params.range.clone());

    let condition = apply_filters(params.filter.clone(), &[("name", super::db::Column::Name)]);

    let (order_column, order_direction) = generic_sort(
        params.sort.clone(),
        &[
            ("id", super::db::Column::Id),
            ("name", super::db::Column::Name),
        ],
        super::db::Column::Id,
    );

    let objs = super::db::Entity::find()
        // .find_with_related(crate::sites::replicates::db::Entity)
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(&db)
        .await
        .unwrap();

    let related = objs
        .load_many(crate::sites::replicates::db::Entity, &db)
        .await
        .unwrap();

    // Map the results from the database models
    let response_objs: Vec<super::models::Site> = objs
        .into_iter()
        .zip(related)
        .map(|(obj, related)| (obj, related).into())
        .collect();

    let total_count: u64 = <super::db::Entity>::find()
        .filter(condition.clone())
        .select_only()
        .column(super::db::Column::Id)
        .count(&db)
        .await
        .unwrap_or(0);

    let headers = calculate_content_range(offset, limit, total_count, RESOURCE_NAME);

    (headers, Json(response_objs))
}

#[utoipa::path(
    get,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Site))
)]
pub async fn get_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<super::models::Site>, (StatusCode, Json<String>)> {
    let obj = match super::db::Entity::find_by_id(id).one(&db).await {
        Ok(Some(obj)) => obj,
        Ok(None) => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
        _ => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
    };

    let related = obj
        .find_related(crate::sites::replicates::db::Entity)
        .all(&db)
        .await
        .unwrap();

    let obj: super::models::Site = (obj, related).into();

    Ok(Json(obj))
}

#[utoipa::path(
    post,
    path = format!("/api/{}", RESOURCE_NAME),
    responses((status = CREATED, body = super::models::Site))
)]
pub async fn create_one(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<super::models::SiteCreate>,
) -> Result<(StatusCode, Json<super::models::Site>), (StatusCode, Json<String>)> {
    let new_obj = super::db::Model {
        id: uuid::Uuid::new_v4(),
        name: payload.name,
        latitude_4326: payload.latitude_4326,
        longitude_4326: payload.longitude_4326,
        elevation_metres: payload.elevation_metres,
    }
    .into_active_model();

    match super::db::Entity::insert(new_obj).exec(&db).await {
        Ok(insert_result) => {
            let response_obj: super::models::Site =
                get_one(State(db.clone()), Path(insert_result.last_insert_id))
                    .await
                    .unwrap()
                    .0;

            Ok((StatusCode::CREATED, Json(response_obj)))
        }
        Err(err) => match err.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                Err((StatusCode::CONFLICT, Json("Duplicate entry".to_string())))
            }
            Some(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error adding object".to_string()),
            )),
            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Server error".to_string()),
            )),
        },
    }
}

#[utoipa::path(
    put,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = OK, body = super::models::Site))
)]
pub async fn update_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<super::models::SiteUpdate>,
) -> impl IntoResponse {
    let db_obj: super::db::ActiveModel = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .unwrap()
        .expect("Failed to find object")
        .into();

    let updated_obj: super::db::ActiveModel = payload.merge_into_activemodel(db_obj);
    let response_obj: super::models::Site = updated_obj.update(&db).await.unwrap().into();

    // Assert response is ok
    assert_eq!(response_obj.id, id);

    // Return the new object
    let obj = get_one(State(db.clone()), Path(id.clone()))
        .await
        .unwrap()
        .0;

    Json(obj)
}

#[utoipa::path(
    delete,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses((status = NO_CONTENT))
)]
pub async fn delete_one(State(db): State<DatabaseConnection>, Path(id): Path<Uuid>) -> StatusCode {
    let obj = super::db::Entity::find_by_id(id)
        .one(&db)
        .await
        .unwrap()
        .expect("Failed to find object");

    let res: DeleteResult = obj.delete(&db).await.expect("Failed to delete object");

    if res.rows_affected == 0 {
        return StatusCode::NOT_FOUND;
    }

    StatusCode::NO_CONTENT
}
