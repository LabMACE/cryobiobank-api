use crudcrate::{CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "isolates")]
#[crudcrate(
    generate_router,
    api_struct = "Isolate",
    name_singular = "isolate",
    name_plural = "isolates",
    description = "Biological isolates with photos and metadata from sample collection sites",
    no_eq,
    derive_partial_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, filterable, exclude(update,create), on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(sortable, filterable)]
    pub site_replicate_id: Uuid,

    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,

    #[crudcrate(sortable, filterable, fulltext)]
    pub taxonomy: Option<String>,
    #[crudcrate(filterable, exclude(list))]
    pub photo: Option<String>,
    #[crudcrate(sortable, filterable)]
    pub temperature_of_isolation: Option<f64>,
    #[crudcrate(sortable, filterable, fulltext)]
    pub media_used_for_isolation: Option<String>,
    #[crudcrate(sortable, filterable, fulltext, exclude(scoped))]
    pub storage_location: Option<String>,
    #[crudcrate(sortable, filterable, fulltext, exclude(scoped))]
    pub description: Option<String>,

    #[crudcrate(sortable, filterable, fulltext)]
    pub genome_url: Option<String>,
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
