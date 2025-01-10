use sea_orm::entity::prelude::*;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "dna")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    pub extraction_method: Option<String>,
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
