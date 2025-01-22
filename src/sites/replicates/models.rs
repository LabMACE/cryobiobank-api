use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::Model;

#[derive(ToSchema, Serialize, Debug)]
pub struct SiteReplicate {
    pub id: Uuid,
    pub name: String,
    pub site_id: Uuid,
    pub sample_type: String,
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

impl From<Model> for SiteReplicate {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            site_id: model.site_id,
            sample_type: model.sample_type,
            sampling_date: model.sampling_date,
            sample_depth_cm: model.sample_depth_cm,
            snow_depth_cm: model.snow_depth_cm,
            air_temperature_celsius: model.air_temperature_celsius,
            snow_temperature_celsius: model.snow_temperature_celsius,
            photosynthetic_active_radiation: model.photosynthetic_active_radiation,
            bacterial_abundance: model.bacterial_abundance,
            cfu_count_r2a: model.cfu_count_r2a,
            cfu_count_another: model.cfu_count_another,
            ph: model.ph,
            ions_fluoride: model.ions_fluoride,
            ions_chloride: model.ions_chloride,
            ions_nitrite: model.ions_nitrite,
            ions_nitrate: model.ions_nitrate,
            ions_bromide: model.ions_bromide,
            ions_sulfate: model.ions_sulfate,
            ions_phosphate: model.ions_phosphate,
            ions_sodium: model.ions_sodium,
            ions_ammonium: model.ions_ammonium,
            ions_potassium: model.ions_potassium,
            ions_magnesium: model.ions_magnesium,
            ions_calcium: model.ions_calcium,
            organic_acids_formate: model.organic_acids_formate,
            organic_acids_malate: model.organic_acids_malate,
            organic_acids_propionate: model.organic_acids_propionate,
            organic_acids_citrate: model.organic_acids_citrate,
            organic_acids_lactate: model.organic_acids_lactate,
            organic_acids_butyrate: model.organic_acids_butyrate,
            organic_acids_oxalate: model.organic_acids_oxalate,
            organic_acids_acetate: model.organic_acids_acetate,
        }
    }
}
