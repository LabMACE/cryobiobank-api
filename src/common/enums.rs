use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SampleType {
    #[sea_orm(string_value = "Snow")]
    Snow,
    #[sea_orm(string_value = "Soil")]
    Soil,
}

impl Default for SampleType {
    fn default() -> Self {
        SampleType::Snow
    }
}

impl fmt::Display for SampleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SampleType::Snow => write!(f, "Snow"),
            SampleType::Soil => write!(f, "Soil"),
        }
    }
}
