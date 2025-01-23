use super::db::{ActiveModel, Model};
use sea_orm::{DeriveIntoActiveModel, IntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Debug)]
pub struct Sample {
    pub id: Uuid,
    pub site_replicate_id: Uuid,
    pub name: String,
    pub type_of_sample: Option<String>,
    pub storage_location: Option<String>,
    pub description: Option<String>,
    pub dna_id: Option<Uuid>,
}

impl From<Model> for Sample {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            site_replicate_id: model.site_replicate_id,
            type_of_sample: model.type_of_sample,
            storage_location: model.storage_location,
            description: model.description,
            dna_id: model.dna_id,
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct SampleCreate {
    pub name: String,
    pub site_replicate_id: Uuid,
    pub type_of_sample: Option<String>,
    pub storage_location: Option<String>,
    pub description: Option<String>,
    pub dna_id: Option<Uuid>,
}

impl From<SampleCreate> for ActiveModel {
    fn from(create: SampleCreate) -> Self {
        super::db::Model {
            id: Uuid::new_v4(),
            site_replicate_id: create.site_replicate_id,
            name: create.name,
            type_of_sample: create.type_of_sample,
            storage_location: create.storage_location,
            description: create.description,
            dna_id: create.dna_id,
        }
        .into_active_model()
    }
}

#[derive(ToSchema, Deserialize)]
pub struct SampleUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub site_replicate_id: Option<Option<Uuid>>,
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
    pub type_of_sample: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub storage_location: Option<Option<String>>,
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
    pub dna_id: Option<Option<Uuid>>,
}

impl SampleUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        model.site_replicate_id = match self.site_replicate_id {
            Some(Some(ref site_replicate_id)) => Set(site_replicate_id.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.name = match self.name {
            Some(Some(ref name)) => Set(name.clone()),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.type_of_sample = match self.type_of_sample {
            Some(Some(ref type_of_sample)) => Set(Some(type_of_sample.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.storage_location = match self.storage_location {
            Some(Some(ref storage_location)) => Set(Some(storage_location.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.description = match self.description {
            Some(Some(ref description)) => Set(Some(description.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.dna_id = match self.dna_id {
            Some(Some(ref dna_id)) => Set(Some(dna_id.clone())),
            Some(_) => Set(None),
            _ => NotSet,
        };

        model
    }
}
