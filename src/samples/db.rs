use crudcrate::{CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, EntityToModels)]
#[sea_orm(table_name = "samples")]
#[crudcrate(
    generate_router,
    api_struct = "Sample",
    name_singular = "sample",
    name_plural = "samples",
    description = "Sample collection records from site replicates with associated DNA",
    derive_partial_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, filterable, exclude(update, create), on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(sortable, filterable)]
    pub site_replicate_id: Uuid,
    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,

    #[crudcrate(sortable, filterable, on_create = true)]
    pub is_available: bool,
    #[crudcrate(sortable, filterable, fulltext, exclude(scoped))]
    pub storage_location: Option<String>,
    #[crudcrate(sortable, filterable, fulltext)]
    pub description: Option<String>,
    #[crudcrate(filterable, exclude(scoped), on_create = false)]
    pub is_private: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::sites::replicates::db::Entity",
        from = "Column::SiteReplicateId",
        to = "crate::sites::replicates::db::Column::Id"
    )]
    SiteReplicate,
}

impl Related<crate::sites::replicates::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SiteReplicate.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
