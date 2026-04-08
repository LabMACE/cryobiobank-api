use chrono::NaiveDate;
use crudcrate::{CRUDResource, EntityToModels};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, EntityToModels)]
#[sea_orm(table_name = "site_replicates")]
#[crudcrate(
    generate_router,
    api_struct = "SiteReplicate",
    name_singular = "site_replicate",
    name_plural = "site_replicates",
    description = "Site replicate sampling points with detailed environmental and chemical data",
    no_eq,
    derive_partial_eq
)]
pub struct Model {
    #[sea_orm(primary_key)]
    #[crudcrate(primary_key, filterable, exclude(update, create), on_create = Uuid::new_v4())]
    pub id: Uuid,
    #[crudcrate(sortable, filterable)]
    pub site_id: Uuid,

    #[sea_orm(unique)]
    #[crudcrate(sortable, filterable, fulltext)]
    pub name: String,

    #[crudcrate(sortable, filterable)]
    pub sampling_date: NaiveDate,
    #[crudcrate(sortable, filterable)]
    pub sample_depth_cm: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub snow_depth_cm: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub air_temperature_celsius: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub snow_temperature_celsius: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub photosynthetic_active_radiation: Option<i32>,
    #[crudcrate(sortable, filterable)]
    pub bacterial_abundance: Option<i64>,
    #[crudcrate(sortable, filterable)]
    pub cfu_count_r2a: Option<i32>,
    #[crudcrate(sortable, filterable)]
    pub cfu_count_another: Option<i32>,
    #[crudcrate(sortable, filterable)]
    pub ph: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_fluoride: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_chloride: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_nitrite: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_nitrate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_bromide: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_sulfate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_phosphate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_sodium: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_ammonium: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_potassium: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_magnesium: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub ions_calcium: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_formate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_malate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_propionate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_citrate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_lactate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_butyrate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_oxalate: Option<f64>,
    #[crudcrate(sortable, filterable)]
    pub organic_acids_acetate: Option<f64>,
    #[crudcrate(filterable, exclude(scoped))]
    pub is_private: bool,
    #[crudcrate(sortable, filterable, fulltext)]
    pub metagenome_url: Option<String>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, exclude(create, update))]
    pub sample_count: Option<i64>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, exclude(create, update))]
    pub isolate_count: Option<i64>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, exclude(create, update))]
    pub dna_count: Option<i64>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, join(one, all, depth = 1))]
    pub samples: Vec<crate::samples::db::Sample>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, join(one, all, depth = 1))]
    pub isolates: Vec<crate::isolates::db::Isolate>,
    #[sea_orm(ignore)]
    #[crudcrate(non_db_attr, join(one, all, depth = 1))]
    pub dna: Vec<crate::dna::db::DNA>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::sites::db::Entity",
        from = "Column::SiteId",
        to = "crate::sites::db::Column::Id"
    )]
    Site,
    #[sea_orm(has_many = "crate::dna::db::Entity")]
    DNA,
    #[sea_orm(has_many = "crate::samples::db::Entity")]
    Samples,
    #[sea_orm(has_many = "crate::isolates::db::Entity")]
    Isolates,
}

impl Related<crate::sites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl Related<crate::dna::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DNA.def()
    }
}

impl Related<crate::samples::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Samples.def()
    }
}

impl Related<crate::isolates::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Isolates.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
