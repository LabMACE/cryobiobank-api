use sea_orm::entity::prelude::*;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "sites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub name: String,
    pub latitude_4326: f64,
    pub longitude_4326: f64,
    pub elevation_metres: f64,
    pub area_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::sites::replicates::db::Entity")]
    SiteReplicates,
    #[sea_orm(has_one = "crate::areas::db::Entity")]
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
