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
    query::*, ActiveModelTrait, DatabaseConnection, DeleteResult, EntityTrait, LoaderTrait,
    ModelTrait, SqlErr,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use uuid::Uuid;

const RESOURCE_NAME: &str = "sites";
#[derive(OpenApi)]
#[openapi(paths(get_all, get_one, create_one, update_one, delete_one, delete_many))]
struct ApiDoc;

pub fn router(
    db: DatabaseConnection,
    keycloak_auth_instance: Option<Arc<KeycloakAuthInstance>>,
) -> Router {
    let mut mutating_router = Router::new()
        .route("/", routing::post(create_one))
        .route("/{id}", routing::put(update_one).delete(delete_one))
        .route("/batch", routing::delete(delete_many))
        .with_state(db.clone());

    if let Some(instance) = keycloak_auth_instance {
        mutating_router = mutating_router.layer(
            KeycloakAuthLayer::<Role>::builder()
                .instance(instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![Role::Administrator])
                .build(),
        );
    } else {
        println!(
            "Warning: Mutating routes of '{}' router are not protected",
            RESOURCE_NAME
        );
    }

    // All the routes that do not mutate the database.
    let router = Router::new()
        // let router = router
        .route("/", routing::get(get_all))
        .route("/{id}", routing::get(get_one))
        .with_state(db.clone())
        .merge(mutating_router)
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()));

    router
}

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
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            ))
        }
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
    let new_obj: super::db::ActiveModel = payload.into();

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

// Deletes a single object
#[utoipa::path(
    delete,
    path = format!("/api/{}/{{id}}", RESOURCE_NAME),
    responses(
        (status = NO_CONTENT, body = Uuid),
        (status = NOT_FOUND, body = String),
        (status = INTERNAL_SERVER_ERROR, body = String)
    )
)]
pub async fn delete_one(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Uuid>), (StatusCode, Json<String>)> {
    let obj = match super::db::Entity::find_by_id(id.clone()).one(&db).await {
        Ok(Some(obj)) => obj,
        Ok(None) => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
        _ => return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string()))),
    };

    let res: DeleteResult = obj.delete(&db).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Failed to delete object".to_string()),
        )
    })?;

    if res.rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json("Not Found".to_string())));
    }

    Ok((StatusCode::NO_CONTENT, Json(id)))
}

#[utoipa::path(
    delete,
    path = format!("/api/{}/batch", RESOURCE_NAME),
    responses(
        (status = NO_CONTENT, body = Vec<Uuid>),
        (status = INTERNAL_SERVER_ERROR, body = String)
    ),
)]
pub async fn delete_many(
    State(db): State<DatabaseConnection>,
    Json(ids): Json<Vec<Uuid>>,
) -> Result<(StatusCode, Json<Vec<Uuid>>), (StatusCode, Json<String>)> {
    let mut deleted_ids = Vec::new();
    for id in ids {
        let obj = match super::db::Entity::find_by_id(id.clone()).one(&db).await {
            Ok(Some(obj)) => obj,
            Ok(None) => continue,
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Failed to delete objects".to_string()),
                ))
            }
        };

        let res: DeleteResult = obj.delete(&db).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed to delete object".to_string()),
            )
        })?;

        if res.rows_affected > 0 {
            deleted_ids.push(id);
        }
    }

    Ok((StatusCode::NO_CONTENT, Json(deleted_ids)))
}
