use chrono::{DateTime, Utc};
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
    #[crudcrate(sortable, filterable, fulltext)]
    pub description: Option<String>,
    #[crudcrate(sortable, filterable, fulltext)]
    pub extraction_method: Option<String>,
    #[crudcrate(filterable, exclude(scoped), on_create = false)]
    pub is_private: bool,
    #[crudcrate(sortable, filterable)]
    pub field_record_id: Uuid,
    #[crudcrate(sortable, filterable, exclude(create, update), on_create = chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::field_records::db::Entity",
        from = "Column::FieldRecordId",
        to = "crate::field_records::db::Column::Id"
    )]
    FieldRecord,
}

impl Related<crate::field_records::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FieldRecord.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
