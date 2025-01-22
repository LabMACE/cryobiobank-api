use sea_orm::{DeriveIntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::{ActiveModel, Model};

#[derive(ToSchema, Serialize, Debug)]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub replicates: Vec<super::replicates::models::SiteReplicate>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

impl From<Model> for Site {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: Vec::new(),
            x: None,
            y: None,
            z: None,
        }
    }
}

impl From<(Model, Vec<super::replicates::db::Model>)> for Site {
    fn from((model, replicates): (Model, Vec<super::replicates::db::Model>)) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: replicates.into_iter().map(|r| r.into()).collect(),
            x: model.x,
            y: model.y,
            z: model.z,
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
    pub x: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub y: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub z: Option<Option<f64>>,
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

        model.x = match self.x {
            Some(Some(ref x)) => Set(Some(x.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };
        model.y = match self.y {
            Some(Some(ref y)) => Set(Some(y.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };
        model.z = match self.z {
            Some(Some(ref z)) => Set(Some(z.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };

        model
    }
}
