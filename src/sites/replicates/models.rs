use super::db::{ActiveModel, Model};
use chrono::NaiveDate;
use sea_orm::{DeriveIntoActiveModel, IntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct SiteReplicate {
    pub id: Uuid,
    pub name: String,
    pub site_id: Uuid,
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
#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct SiteReplicateCreate {
    pub name: String,
    pub site_id: Uuid,
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

impl From<SiteReplicateCreate> for ActiveModel {
    fn from(create: SiteReplicateCreate) -> Self {
        super::db::Model {
            id: Uuid::new_v4(),
            name: create.name,
            site_id: create.site_id,
            sample_type: create.sample_type,
            sampling_date: create.sampling_date,
            sample_depth_cm: create.sample_depth_cm,
            snow_depth_cm: create.snow_depth_cm,
            air_temperature_celsius: create.air_temperature_celsius,
            snow_temperature_celsius: create.snow_temperature_celsius,
            photosynthetic_active_radiation: create.photosynthetic_active_radiation,
            bacterial_abundance: create.bacterial_abundance,
            cfu_count_r2a: create.cfu_count_r2a,
            cfu_count_another: create.cfu_count_another,
            ph: create.ph,
            ions_fluoride: create.ions_fluoride,
            ions_chloride: create.ions_chloride,
            ions_nitrite: create.ions_nitrite,
            ions_nitrate: create.ions_nitrate,
            ions_bromide: create.ions_bromide,
            ions_sulfate: create.ions_sulfate,
            ions_phosphate: create.ions_phosphate,
            ions_sodium: create.ions_sodium,
            ions_ammonium: create.ions_ammonium,
            ions_potassium: create.ions_potassium,
            ions_magnesium: create.ions_magnesium,
            ions_calcium: create.ions_calcium,
            organic_acids_formate: create.organic_acids_formate,
            organic_acids_malate: create.organic_acids_malate,
            organic_acids_propionate: create.organic_acids_propionate,
            organic_acids_citrate: create.organic_acids_citrate,
            organic_acids_lactate: create.organic_acids_lactate,
            organic_acids_butyrate: create.organic_acids_butyrate,
            organic_acids_oxalate: create.organic_acids_oxalate,
            organic_acids_acetate: create.organic_acids_acetate,
        }
        .into_active_model()
    }
}

#[derive(ToSchema, Deserialize)]
pub struct SiteReplicateUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub site_id: Option<Option<Uuid>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub sample_type: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[schema(value_type = String, format = Date)]
    pub sampling_date: Option<Option<NaiveDate>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub sample_depth_cm: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub snow_depth_cm: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub air_temperature_celsius: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub snow_temperature_celsius: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub photosynthetic_active_radiation: Option<Option<i32>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub bacterial_abundance: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub cfu_count_r2a: Option<Option<i32>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub cfu_count_another: Option<Option<i32>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ph: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_fluoride: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_chloride: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_nitrite: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_nitrate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_bromide: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_sulfate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_phosphate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_sodium: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_ammonium: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_potassium: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_magnesium: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ions_calcium: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_formate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_malate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_propionate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_citrate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_lactate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_butyrate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_oxalate: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub organic_acids_acetate: Option<Option<f64>>,
}

impl SiteReplicateUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        model.name = match self.name {
            Some(Some(ref name)) => Set(name.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.site_id = match self.site_id {
            Some(Some(ref site_id)) => Set(site_id.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.sample_type = match self.sample_type {
            Some(Some(ref sample_type)) => Set(sample_type.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.sampling_date = match self.sampling_date {
            Some(Some(ref sampling_date)) => Set(sampling_date.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.sample_depth_cm = match self.sample_depth_cm {
            Some(Some(ref sample_depth_cm)) => Set(Some(sample_depth_cm.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.snow_depth_cm = match self.snow_depth_cm {
            Some(Some(ref snow_depth_cm)) => Set(Some(snow_depth_cm.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.air_temperature_celsius = match self.air_temperature_celsius {
            Some(Some(ref air_temperature_celsius)) => Set(Some(air_temperature_celsius.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.snow_temperature_celsius = match self.snow_temperature_celsius {
            Some(Some(ref snow_temperature_celsius)) => Set(Some(snow_temperature_celsius.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.photosynthetic_active_radiation = match self.photosynthetic_active_radiation {
            Some(Some(ref photosynthetic_active_radiation)) => {
                Set(Some(photosynthetic_active_radiation.clone()))
            }
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.bacterial_abundance = match self.bacterial_abundance {
            Some(Some(ref bacterial_abundance)) => Set(Some(bacterial_abundance.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.cfu_count_r2a = match self.cfu_count_r2a {
            Some(Some(ref cfu_count_r2a)) => Set(Some(cfu_count_r2a.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.cfu_count_another = match self.cfu_count_another {
            Some(Some(ref cfu_count_another)) => Set(Some(cfu_count_another.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ph = match self.ph {
            Some(Some(ref ph)) => Set(Some(ph.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_fluoride = match self.ions_fluoride {
            Some(Some(ref ions_fluoride)) => Set(Some(ions_fluoride.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_chloride = match self.ions_chloride {
            Some(Some(ref ions_chloride)) => Set(Some(ions_chloride.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_nitrite = match self.ions_nitrite {
            Some(Some(ref ions_nitrite)) => Set(Some(ions_nitrite.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_nitrate = match self.ions_nitrate {
            Some(Some(ref ions_nitrate)) => Set(Some(ions_nitrate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_bromide = match self.ions_bromide {
            Some(Some(ref ions_bromide)) => Set(Some(ions_bromide.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_sulfate = match self.ions_sulfate {
            Some(Some(ref ions_sulfate)) => Set(Some(ions_sulfate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_phosphate = match self.ions_phosphate {
            Some(Some(ref ions_phosphate)) => Set(Some(ions_phosphate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_sodium = match self.ions_sodium {
            Some(Some(ref ions_sodium)) => Set(Some(ions_sodium.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_ammonium = match self.ions_ammonium {
            Some(Some(ref ions_ammonium)) => Set(Some(ions_ammonium.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_potassium = match self.ions_potassium {
            Some(Some(ref ions_potassium)) => Set(Some(ions_potassium.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_magnesium = match self.ions_magnesium {
            Some(Some(ref ions_magnesium)) => Set(Some(ions_magnesium.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.ions_calcium = match self.ions_calcium {
            Some(Some(ref ions_calcium)) => Set(Some(ions_calcium.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_formate = match self.organic_acids_formate {
            Some(Some(ref organic_acids_formate)) => Set(Some(organic_acids_formate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_malate = match self.organic_acids_malate {
            Some(Some(ref organic_acids_malate)) => Set(Some(organic_acids_malate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_propionate = match self.organic_acids_propionate {
            Some(Some(ref organic_acids_propionate)) => Set(Some(organic_acids_propionate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_citrate = match self.organic_acids_citrate {
            Some(Some(ref organic_acids_citrate)) => Set(Some(organic_acids_citrate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_lactate = match self.organic_acids_lactate {
            Some(Some(ref organic_acids_lactate)) => Set(Some(organic_acids_lactate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_butyrate = match self.organic_acids_butyrate {
            Some(Some(ref organic_acids_butyrate)) => Set(Some(organic_acids_butyrate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_oxalate = match self.organic_acids_oxalate {
            Some(Some(ref organic_acids_oxalate)) => Set(Some(organic_acids_oxalate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.organic_acids_acetate = match self.organic_acids_acetate {
            Some(Some(ref organic_acids_acetate)) => Set(Some(organic_acids_acetate.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model
    }
}
