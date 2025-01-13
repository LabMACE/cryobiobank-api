use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use super::db::Model;

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
