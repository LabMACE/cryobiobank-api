use sea_orm::{DeriveIntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::{ActiveModel, Model};

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub replicates: Vec<super::replicates::models::SiteReplicate>,
    pub longitude_4326: Option<f64>,
    pub latitude_4326: Option<f64>,
    pub elevation_metres: Option<f64>,
}

impl From<Model> for Site {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: Vec::new(),
            longitude_4326: None,
            latitude_4326: None,
            elevation_metres: None,
        }
    }
}

impl From<(Model, Vec<super::replicates::db::Model>)> for Site {
    fn from((model, replicates): (Model, Vec<super::replicates::db::Model>)) -> Self {
        println!("{:?}", model);
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
    // pub geometry: String,
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
            Some(Some(ref longitude_4326)) => Set(Some(longitude_4326.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };
        model.latitude_4326 = match self.latitude_4326 {
            Some(Some(ref latitude_4326)) => Set(Some(latitude_4326.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };
        model.elevation_metres = match self.elevation_metres {
            Some(Some(ref elevation_metres)) => Set(Some(elevation_metres.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };

        model
    }
}
