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
    no_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, exclude(update,create), on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(sortable, filterable)]
    pub site_replicate_id: Uuid,

    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,

    #[crudcrate(sortable, filterable, fulltext)]
    pub taxonomy: Option<String>,
    #[crudcrate(filterable)]
    pub photo: Option<String>,
    #[crudcrate(sortable, filterable)]
    pub temperature_of_isolation: Option<f64>,
    #[crudcrate(sortable, filterable, fulltext)]
    pub media_used_for_isolation: Option<String>,
    #[crudcrate(sortable, filterable, fulltext)]
    pub storage_location: Option<String>,

    #[crudcrate(sortable, filterable)]
    pub dna_id: Option<Uuid>,
    #[crudcrate(filterable)]
    pub is_private: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::dna::db::Entity",
        from = "Column::DnaId",
        to = "crate::dna::db::Column::Id"
    )]
    DNA,
    #[sea_orm(
        belongs_to = "crate::sites::replicates::db::Entity",
        from = "Column::SiteReplicateId",
        to = "crate::sites::replicates::db::Column::Id"
    )]
    SiteReplicate,
}

// `Related` trait has to be implemented by hand
impl Related<crate::dna::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DNA.def()
    }
}

impl Related<crate::sites::replicates::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SiteReplicate.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
