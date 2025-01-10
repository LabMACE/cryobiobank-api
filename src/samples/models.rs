use sea_orm::DeriveIntoActiveModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::{ActiveModel, Model};

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

/// Creating new samples
#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct SampleCreate {
    pub name: String,
    pub site_replicate_id: Uuid,
    pub type_of_sample: Option<String>,
    pub storage_location: Option<String>,
    pub description: Option<String>,
    pub dna_id: Option<Uuid>,
}
