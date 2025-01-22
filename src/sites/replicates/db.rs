use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, ToSchema)]
#[sea_orm(table_name = "site_replicates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub site_id: Uuid,

    #[sea_orm(unique)]
    pub name: String,

    #[sea_orm(column_name = "sample_type")]
    pub sample_type: String,
    #[schema(value_type = String, format = Date)]
    pub sampling_date: NaiveDate,
    pub sample_depth_cm: Option<f64>,
    pub snow_depth_cm: Option<f64>,
    pub air_temperature_celsius: Option<f64>,
    pub snow_temperature_celsius: Option<f64>,
    pub photosynthetic_active_radiation: Option<i32>,
    pub bacterial_abundance: Option<i64>,
    pub cfu_count_r2a: Option<i32>,
    pub cfu_count_another: Option<i32>,

    pub ph: Option<f64>,
    pub ions_fluoride: Option<f64>,
    pub ions_chloride: Option<f64>,
    pub ions_nitrite: Option<f64>,
    pub ions_nitrate: Option<f64>,
    pub ions_bromide: Option<f64>,
    pub ions_sulfate: Option<f64>,
    pub ions_phosphate: Option<f64>,
    pub ions_sodium: Option<f64>,
    pub ions_ammonium: Option<f64>,
    pub ions_potassium: Option<f64>,
    pub ions_magnesium: Option<f64>,
    pub ions_calcium: Option<f64>,
    pub organic_acids_formate: Option<f64>,
    pub organic_acids_malate: Option<f64>,
    pub organic_acids_propionate: Option<f64>,
    pub organic_acids_citrate: Option<f64>,
    pub organic_acids_lactate: Option<f64>,
    pub organic_acids_butyrate: Option<f64>,
    pub organic_acids_oxalate: Option<f64>,
    pub organic_acids_acetate: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::sites::db::Entity",
        from = "Column::SiteId",
        to = "crate::sites::db::Column::Id"
    )]
    Site,
}

impl Related<crate::sites::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
