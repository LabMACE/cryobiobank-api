use super::db::{ActiveModel, Model};
use sea_orm::{DeriveIntoActiveModel, IntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub replicates: Vec<super::replicates::models::SiteReplicate>,
    pub longitude_4326: f64,
    pub latitude_4326: f64,
    pub elevation_metres: f64,
}

impl From<Model> for Site {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: Vec::new(),
            longitude_4326: model.longitude_4326,
            latitude_4326: model.latitude_4326,
            elevation_metres: model.elevation_metres,
        }
    }
}

impl From<(Model, Vec<super::replicates::db::Model>)> for Site {
    fn from((model, replicates): (Model, Vec<super::replicates::db::Model>)) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: replicates.into_iter().map(|r| r.into()).collect(),
            longitude_4326: model.longitude_4326,
            latitude_4326: model.latitude_4326,
            elevation_metres: model.elevation_metres,
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct SiteCreate {
    pub name: String,
    pub longitude_4326: f64,
    pub latitude_4326: f64,
    pub elevation_metres: f64,
}

impl From<SiteCreate> for ActiveModel {
    fn from(create: SiteCreate) -> Self {
        super::db::Model {
            id: Uuid::new_v4(),
            name: create.name,
            longitude_4326: create.longitude_4326,
            latitude_4326: create.latitude_4326,
            elevation_metres: create.elevation_metres,
        }
        .into_active_model()
    }
}

#[derive(ToSchema, Deserialize)]
pub struct SiteUpdate {
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
    pub longitude_4326: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub latitude_4326: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub elevation_metres: Option<Option<f64>>,
}
impl SiteUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        // If the field is Some(None), update the field to None, if None,
        // do not update the field (double option)

        model.name = match self.name {
            Some(Some(ref name)) => Set(name.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.longitude_4326 = match self.longitude_4326 {
            Some(Some(ref longitude_4326)) => Set(longitude_4326.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };
        model.latitude_4326 = match self.latitude_4326 {
            Some(Some(ref latitude_4326)) => Set(latitude_4326.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };
        model.elevation_metres = match self.elevation_metres {
            Some(Some(ref elevation_metres)) => Set(elevation_metres.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model
    }
}
