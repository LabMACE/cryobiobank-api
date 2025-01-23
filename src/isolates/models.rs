use super::db::{ActiveModel, Model};
use sea_orm::{DeriveIntoActiveModel, IntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(ToSchema, Serialize, Debug)]
pub struct Isolate {
    pub id: Uuid,
    pub site_replicate_id: Uuid,
    pub name: String,
    pub taxonomy: Option<String>,
    pub photo: Option<String>,
    pub temperature_of_isolation: Option<f64>,
    pub media_used_for_isolation: Option<String>,
    pub storage_location: Option<String>,
    pub dna_id: Option<Uuid>,
}

impl From<Model> for Isolate {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            taxonomy: model.taxonomy,
            photo: model.photo,
            site_replicate_id: model.site_replicate_id,
            temperature_of_isolation: model.temperature_of_isolation,
            media_used_for_isolation: model.media_used_for_isolation,
            storage_location: model.storage_location,
            dna_id: model.dna_id,
        }
    }
}
#[derive(ToSchema, Deserialize, Serialize, DeriveIntoActiveModel)]
pub struct IsolateCreate {
    pub site_replicate_id: Uuid,
    pub name: String,
    pub taxonomy: Option<String>,
    pub photo: Option<String>,
    pub temperature_of_isolation: Option<f64>,
    pub media_used_for_isolation: Option<String>,
    pub storage_location: Option<String>,
    pub dna_id: Option<Uuid>,
}

impl From<IsolateCreate> for ActiveModel {
    fn from(create: IsolateCreate) -> Self {
        super::db::Model {
            id: Uuid::new_v4(),
            site_replicate_id: create.site_replicate_id,
            name: create.name,
            taxonomy: create.taxonomy,
            photo: create.photo,
            temperature_of_isolation: create.temperature_of_isolation,
            media_used_for_isolation: create.media_used_for_isolation,
            storage_location: create.storage_location,
            dna_id: create.dna_id,
        }
        .into_active_model()
    }
}

#[derive(ToSchema, Deserialize)]
pub struct IsolateUpdate {
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
    pub taxonomy: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub photo: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub temperature_of_isolation: Option<Option<f64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub media_used_for_isolation: Option<Option<String>>,
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
    pub dna_id: Option<Option<Uuid>>,
}

impl IsolateUpdate {
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

        model.taxonomy = match self.taxonomy {
            Some(Some(ref taxonomy)) => Set(Some(taxonomy.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.photo = match self.photo {
            Some(Some(ref photo)) => Set(Some(photo.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.temperature_of_isolation = match self.temperature_of_isolation {
            Some(Some(ref temperature_of_isolation)) => Set(Some(*temperature_of_isolation)),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.media_used_for_isolation = match self.media_used_for_isolation {
            Some(Some(ref media_used_for_isolation)) => Set(Some(media_used_for_isolation.clone())),
            Some(_) => NotSet,
            _ => NotSet,
        };

        model.storage_location = match self.storage_location {
            Some(Some(ref storage_location)) => Set(Some(storage_location.clone())),
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
