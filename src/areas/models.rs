use super::db::{ActiveModel, Model};
use rand::Rng;
use sea_orm::{IntoActiveModel, NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(ToSchema, Serialize, Deserialize, Debug, Validate)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
    pub geom: Option<serde_json::Value>,
}

impl From<Model> for Area {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            colour: Some(model.colour),
            geom: None,
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize, Validate)]
pub struct AreaCreate {
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
}

impl From<AreaCreate> for ActiveModel {
    fn from(create: AreaCreate) -> Self {
        let colour = create.colour.unwrap_or_else(|| {
            let mut rng = rand::rng();
            format!("#{:06x}", rng.random::<u32>() & 0xFFFFFF)
        });

        super::db::Model {
            id: Uuid::new_v4(),
            name: create.name,
            description: create.description,
            colour,
        }
        .into_active_model()
    }
}

#[derive(ToSchema, Deserialize, Validate)]
pub struct AreaUpdate {
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
    pub colour: Option<Option<String>>,
}

impl AreaUpdate {
    pub fn merge_into_activemodel(&self, mut model: ActiveModel) -> ActiveModel {
        model.name = match &self.name {
            Some(Some(ref name)) => Set(name.clone()),
            Some(_) => NotSet,
            None => NotSet,
        };
        model.description = match &self.description {
            Some(Some(ref description)) => Set(Some(description.clone())),
            Some(_) => NotSet,
            None => NotSet,
        };
        model.colour = match &self.colour {
            Some(Some(ref colour)) => Set(colour.clone()),
            Some(_) => NotSet,
            None => NotSet,
        };
        model
    }
}
