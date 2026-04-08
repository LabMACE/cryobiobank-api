use crudcrate::{ApiError, CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, DatabaseConnection, Order, QueryOrder, QuerySelect};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "areas")]
#[crudcrate(
    generate_router,
    api_struct = "Area",
    name_singular = "area",
    name_plural = "areas",
    description = "Geographic areas for organizing collection sites with convex hull geometry",
    read::many::body = get_all_areas_with_geometry
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, filterable, exclude(update, create), on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,
    #[crudcrate(sortable, filterable, fulltext)]
    pub description: Option<String>,
    #[crudcrate(sortable, filterable)]
    pub colour: String,
    #[crudcrate(filterable, exclude(scoped))]
    pub is_private: bool,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr)]
    pub geom: Option<serde_json::Value>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, join(one, depth = 1))]
    pub sites: Vec<crate::sites::db::Site>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::sites::db::Entity")]
    Site,
}

impl Related<crate::sites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Custom get_all function that includes convex hull geometry
pub(super) async fn get_all_areas_with_geometry(
    db: &DatabaseConnection,
    condition: &Condition,
    order_column: Column,
    order_direction: Order,
    offset: u64,
    limit: u64,
) -> Result<Vec<AreaList>, ApiError> {
    let models = Entity::find()
        .filter(condition.clone())
        .order_by(order_column, order_direction)
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;

    // Batch-fetch all convex hulls in one PostGIS query (instead of N+1)
    let area_ids: Vec<Uuid> = models.iter().map(|m| m.id).collect();
    let hulls = crate::areas::services::get_convex_hulls_batch(db, &area_ids).await;

    let areas = models
        .into_iter()
        .map(|model| {
            let geom = hulls.get(&model.id).cloned();
            let mut area: AreaList = model.into();
            area.geom = geom;
            area
        })
        .collect();

    Ok(areas)
}
