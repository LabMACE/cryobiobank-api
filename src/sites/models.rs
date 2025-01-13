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
    // pub geometry: String,
}

impl From<Model> for Site {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: Vec::new(),
            // geometry: model.geometry,
        }
    }
}

impl From<(Model, Vec<super::replicates::db::Model>)> for Site {
    fn from((model, replicates): (Model, Vec<super::replicates::db::Model>)) -> Self {
        Self {
            id: model.id,
            name: model.name,
            replicates: replicates.into_iter().map(|r| r.into()).collect(),
            // geometry: model.geometry,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub geometry: Option<String>,
}

impl From<SiteUpdate> for ActiveModel {
    fn from(update: SiteUpdate) -> Self {
        Self {
            name: update.name.map(Set).unwrap_or(NotSet),
            // geometry: update.geometry.map(Set).unwrap_or(NotSet),
            id: NotSet,
        }
    }
}
