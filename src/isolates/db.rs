use sea_orm::entity::prelude::*;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "isolates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub site_replicate_id: Uuid,

    #[sea_orm(unique)]
    pub name: String,

    pub taxonomy: Option<String>,
    pub photo: Option<String>,
    pub temperature_of_isolation: Option<f64>,
    pub media_used_for_isolation: Option<String>,
    pub storage_location: Option<String>,

    pub dna_id: Option<Uuid>,
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
