use crudcrate::{CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "dna")]
#[crudcrate(
    generate_router,
    api_struct = "DNA",
    name_singular = "dna",
    name_plural = "dna",
    description = "DNA extraction and analysis records for biological samples",
    no_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, exclude(update, create), on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,
    #[crudcrate(sortable, filterable, fulltext)]
    pub description: Option<String>,
    #[crudcrate(sortable, filterable, fulltext)]
    pub extraction_method: Option<String>,
    #[crudcrate(filterable)]
    pub is_private: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::isolates::db::Entity")]
    Isolates,
    #[sea_orm(has_many = "crate::samples::db::Entity")]
    Samples,
}

impl Related<crate::isolates::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Isolates.def()
    }
}

impl Related<crate::samples::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Samples.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
