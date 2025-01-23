use super::db::{ActiveModel, Model};
use sea_orm::{DeriveIntoActiveModel, IntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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

#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct DNACreate {
    pub name: String,
    pub description: Option<String>,
    pub extraction_method: Option<String>,
}

impl From<DNACreate> for ActiveModel {
    fn from(create: DNACreate) -> Self {
        super::db::Model {
            id: Uuid::new_v4(),
            name: create.name,
            description: create.description,
            extraction_method: create.extraction_method,
        }
        .into_active_model()
    }
}

#[derive(ToSchema, Deserialize)]
pub struct DNAUpdate {
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
    pub description: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub extraction_method: Option<Option<String>>,
}

impl DNAUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        // If the field is Some(None), update the field to None, if None,
        // do not update the field (double option)

        model.name = match self.name {
            Some(Some(ref name)) => Set(name.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.description = match self.description {
            Some(Some(ref description)) => Set(Some(description.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.extraction_method = match self.extraction_method {
            Some(Some(ref extraction_method)) => Set(Some(extraction_method.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model
    }
}
