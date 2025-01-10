use sea_orm::{DeriveIntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::{ActiveModel, Model};

#[derive(ToSchema, Serialize, Debug)]
pub struct DNA {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub extraction_method: Option<String>,
}

impl From<Model> for DNA {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            extraction_method: model.extraction_method,
        }
    }
}

/// For creating a new DNA record
#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct DNACreate {
    pub name: String,
    pub description: Option<String>,
    pub extraction_method: Option<String>,
}

/// For partial updates
#[derive(ToSchema, Deserialize)]
pub struct DNAUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extraction_method: Option<String>,
}

impl From<DNAUpdate> for ActiveModel {
    fn from(update: DNAUpdate) -> Self {
        Self {
            name: update.name.map(Set).unwrap_or(NotSet),
            description: update
                .description
                .map(|desc| Set(Some(desc)))
                .unwrap_or(NotSet),
            extraction_method: update
                .extraction_method
                .map(|em| Set(Some(em)))
                .unwrap_or(NotSet),
            // Primary key and other fields not updated remain NotSet
            id: NotSet,
        }
    }
}

impl DNAUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        if let Some(name) = &self.name {
            model.name = Set(name.clone());
        }
        if let Some(description) = &self.description {
            model.description = Set(Some(description.clone()));
        }
        if let Some(extraction_method) = &self.extraction_method {
            model.extraction_method = Set(Some(extraction_method.clone()));
        }

        model
    }
}
