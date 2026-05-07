use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[derive(Default)]
pub enum SampleType {
    #[sea_orm(string_value = "Snow")]
    #[default]
    Snow,
    #[sea_orm(string_value = "Soil")]
    Soil,
}


impl fmt::Display for SampleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SampleType::Snow => write!(f, "Snow"),
            SampleType::Soil => write!(f, "Soil"),
        }
    }
}
