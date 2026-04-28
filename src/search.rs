use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use crudcrate::filtering::build_fulltext_condition;
use crudcrate::CRUDResource;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::middleware;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: HashMap<String, Vec<serde_json::Value>>,
    pub total: usize,
}

async fn search_resource<T: CRUDResource>(
    q: &str,
    backend: DatabaseBackend,
    db: &DatabaseConnection,
    scope: Option<sea_orm::Condition>,
) -> Vec<serde_json::Value>
where
    <T::EntityType as EntityTrait>::Column: From<T::ColumnType>,
{
    let Some(cond) = build_fulltext_condition::<T>(q, backend) else {
        return vec![];
    };
    let name_col = T::fulltext_searchable_columns()
        .first()
        .map(|(_, col)| *col);

    let mut select = T::EntityType::find()
        .select_only()
        .column(T::ID_COLUMN)
        .filter(cond)
        .limit(5);

    if let Some(scope_cond) = scope {
        select = select.filter(scope_cond);
    }

    if let Some(col) = name_col {
        select = select.column(col).order_by_asc(col);
    }

    select.into_json().all(db).await.unwrap_or_default()
}

pub async fn search(
    State(db): State<DatabaseConnection>,
    Query(params): Query<SearchParams>,
    req: Request,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let query = params.q.trim().to_string();

    if query.len() < 2 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Search query must be at least 2 characters".to_string(),
        ));
    }

    if query.len() > 200 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Search query must be at most 200 characters".to_string(),
        ));
    }

    let backend = db.get_database_backend();

    let scope_public = !middleware::is_admin(&req);
    let area_scope = scope_public.then(middleware::areas_scope);
    let site_scope = scope_public.then(middleware::sites_scope);
    let field_record_scope = scope_public.then(middleware::field_records_scope);
    let isolate_scope = scope_public.then(middleware::isolates_scope);
    let sample_scope = scope_public.then(middleware::samples_scope);
    let dna_scope = scope_public.then(middleware::dna_scope);

    use crate::{areas, dna, field_records, isolates, samples, sites};

    let (areas, sites, field_records, isolates, samples, dna) = tokio::join!(
        search_resource::<areas::db::Area>(&query, backend, &db, area_scope),
        search_resource::<sites::db::Site>(&query, backend, &db, site_scope),
        search_resource::<field_records::db::FieldRecord>(&query, backend, &db, field_record_scope),
        search_resource::<isolates::db::Isolate>(&query, backend, &db, isolate_scope),
        search_resource::<samples::db::Sample>(&query, backend, &db, sample_scope),
        search_resource::<dna::db::DNA>(&query, backend, &db, dna_scope),
    );

    let total = areas.len()
        + sites.len()
        + field_records.len()
        + isolates.len()
        + samples.len()
        + dna.len();

    let results = HashMap::from([
        ("areas".to_string(), areas),
        ("sites".to_string(), sites),
        ("field_records".to_string(), field_records),
        ("isolates".to_string(), isolates),
        ("samples".to_string(), samples),
        ("dna".to_string(), dna),
    ]);

    Ok(Json(SearchResponse {
        query,
        results,
        total,
    }))
}
