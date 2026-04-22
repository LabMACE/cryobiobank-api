use crudcrate::ApiError;
use sea_orm::entity::prelude::*;
use sea_orm::{
    Condition, DatabaseConnection, DbBackend, Order, QueryOrder, QuerySelect, Statement,
};
use std::collections::HashSet;
use uuid::Uuid;

use super::db::{Column, Entity, IsolateList};

/// Custom list hook — runs the default scoped query, then batches a second
/// lookup to flag which isolates have a non-empty photo. Keeps `photo` out of
/// the list payload (still `exclude(list)`) while letting the UI show a
/// "photo available" cue without per-card detail fetches.
pub(super) async fn get_all_isolates_with_photo_flag(
    db: &DatabaseConnection,
    condition: &Condition,
    order_column: Column,
    order_direction: Order,
    offset: u64,
    limit: u64,
) -> Result<Vec<IsolateList>, ApiError> {
    let models = Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    let ids: Vec<Uuid> = models.iter().map(|m| m.id).collect();
    let with_photo = photo_id_set(db, &ids).await;

    Ok(models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let mut list: IsolateList = m.into();
            list.has_photo = with_photo.contains(&id);
            list
        })
        .collect())
}

async fn photo_id_set(db: &DatabaseConnection, ids: &[Uuid]) -> HashSet<Uuid> {
    if ids.is_empty() {
        return HashSet::new();
    }

    let rows = match db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM isolates WHERE id = ANY($1) AND photo IS NOT NULL AND photo <> ''",
            vec![ids.to_vec().into()],
        ))
        .await
    {
        Ok(rows) => rows,
        Err(_) => return HashSet::new(),
    };

    rows.into_iter()
        .filter_map(|r| r.try_get::<Uuid>("", "id").ok())
        .collect()
}
