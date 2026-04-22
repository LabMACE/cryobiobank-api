use crudcrate::{CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "sites")]
#[crudcrate(
    generate_router,
    api_struct = "Site",
    name_singular = "site",
    name_plural = "sites",
    description = "Collection sites with coordinates and associated site replicates",
    no_eq,
    derive_partial_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, filterable, exclude(update, create), on_create = Uuid::new_v4())]
    pub id: Uuid,

    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,
    #[crudcrate(sortable, filterable)]
    pub latitude_4326: f64,
    #[crudcrate(sortable, filterable)]
    pub longitude_4326: f64,
    #[crudcrate(sortable, filterable)]
    pub elevation_metres: f64,
    #[crudcrate(sortable, filterable)]
    pub area_id: Option<Uuid>,
    #[crudcrate(filterable, exclude(scoped), on_create = false)]
    pub is_private: bool,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, exclude(create, update), join(one, all, depth = 2))]
    pub replicates: Vec<crate::sites::replicates::db::SiteReplicate>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::sites::replicates::db::Entity")]
    SiteReplicates,
    #[sea_orm(
        belongs_to = "crate::areas::db::Entity",
        from = "Column::AreaId",
        to = "crate::areas::db::Column::Id"
    )]
    Area,
}

impl Related<crate::sites::replicates::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SiteReplicates.def()
    }
}

impl Related<crate::areas::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Area.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
