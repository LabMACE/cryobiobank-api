//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "submissions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: Option<String>,
    pub processing_has_started: Option<bool>,
    pub processing_success: Option<bool>,
    pub comment: Option<String>,
    pub created_on: Option<DateTime>,
    pub last_updated: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::file_object_associations::Entity")]
    FileObjectAssociations,
}

impl Related<super::file_object_associations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FileObjectAssociations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
