use crate::common::enums::SampleType;
use crate::config::Config;
use sea_orm::Iterable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(ToSchema, Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

#[derive(ToSchema, Deserialize, Serialize, Default)]
pub struct Keycloak {
    pub client_id: String,
    pub realm: String,
    pub url: String,
}

#[derive(ToSchema, Deserialize, Serialize, Default)]
pub struct UIConfiguration {
    pub keycloak: Keycloak,
    pub deployment: String,
    pub sample_types: Vec<String>,
}

impl UIConfiguration {
    pub fn new() -> Self {
        let config: Config = Config::from_env();
        Self {
            keycloak: Keycloak {
                client_id: config.keycloak_ui_id,
                realm: config.keycloak_realm,
                url: config.keycloak_url,
            },
            deployment: config.deployment,
            sample_types: SampleType::iter().map(|st| st.to_string()).collect(),
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct HealthCheck {
    pub status: String,
}

#[derive(ToSchema, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct ServiceStatus {
    pub s3_status: bool,
    pub kubernetes_status: bool,
}
