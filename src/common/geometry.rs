// src/common/geometry.rs

use std::fmt;

use geozero::wkb::{Decode, Ewkb};
use sea_orm::entity::prelude::*; // For QueryResult, TryGetError, etc.
use sea_orm::sea_query::{ArrayType, ColumnType, Value, ValueType, ValueTypeErr};
use sea_orm::{ColIdx, FromQueryResult, TryGetable, TryGetableMany}; // geozero's WKB/EWKB support.

// For demonstration, let's also show how to convert from/to geo-types:
use geo_types::Geometry as GeoGeometry;
use geozero::{GeozeroDatasource, GeozeroGeometry};

/// A newtype holding the raw EWKB bytes from a PostGIS geometry column.
///
/// `None` means the geometry column was NULL.
#[derive(Clone, Debug, PartialEq)]
pub struct WkbGeometry(pub Option<Vec<u8>>);

impl WkbGeometry {
    /// If we have geometry bytes, return them as `geozero::wkb::Ewkb`.
    pub fn as_ewkb(&self) -> Option<Ewkb<Vec<u8>>> {
        self.0.as_ref().map(|b| Ewkb(b.clone()))
    }

    /// A helper to parse the geometry bytes into a `geo_types::Geometry<f64>`.
    /// If it's `None`, we return `Ok(None)`.
    pub fn to_geo(&self) -> Result<Option<GeoGeometry<f64>>, String> {
        let Some(raw) = &self.0 else {
            return Ok(None); // If column was NULL
        };
        let ewkb = Ewkb(raw.clone()); // geozero's EWKB wrapper

        // We'll use geozero's `Decode<geo_types::Geometry<f64>>`.
        // This struct can parse the EWKB into a `geo_types::Geometry`.
        let mut decoder = Decode::<GeoGeometry<f64>> { geometry: None };
        decoder
            .process_ewkb(&mut &raw[..]) // read from the bytes
            .map_err(|e| e.to_string())?;

        Ok(decoder.geometry)
    }
}

/// Let's implement `fmt::Display` for debugging/logging.
impl fmt::Display for WkbGeometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(bytes) => write!(f, "WkbGeometry({} bytes)", bytes.len()),
            None => write!(f, "WkbGeometry(NULL)"),
        }
    }
}

impl From<WkbGeometry> for Value {
    fn from(src: WkbGeometry) -> Self {
        // Convert to Value::Bytes
        match src.0 {
            Some(bytes) => Value::Bytes(Some(Box::new(bytes))),
            None => Value::Bytes(None),
        }
    }
}

impl TryGetable for WkbGeometry {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        // The underlying column is "geometry", but from Postgres
        // we actually get raw EWKB as `bytea`.
        let bytes_opt: Option<Vec<u8>> = <Option<Vec<u8>>>::try_get_by(res, idx)?;
        Ok(WkbGeometry(bytes_opt))
    }
}

// // For some queries, SeaORM calls `TryGetableMany`
// impl TryGetableMany for WkbGeometry {
//     fn try_get_many(res: &QueryResult, pre: &str, col: &str) -> Result<Vec<Self>, TryGetError> {
//         let bytes_opt: Option<Vec<u8>> = <Option<Vec<u8>>>::try_get_many(res, pre, col)?;
//         Ok(bytes_opt.into_iter().map(WkbGeometry).collect())
//     }

//     fn try_get_many_by_index(res: &QueryResult, idx: usize) -> Result<Vec<Self>, TryGetError> {
//         let bytes_opt: Option<Vec<u8>> = <Option<Vec<u8>>>::try_get_many_by_index(res, idx)?;
//         Ok(bytes_opt.into_iter().map(WkbGeometry).collect())
//     }
// }

impl ValueType for WkbGeometry {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Bytes(Some(bytes)) => Ok(WkbGeometry(Some(*bytes))),
            Value::Bytes(None) => Ok(WkbGeometry(None)),
            _ => Err(ValueTypeErr::Custom(
                "Expected Bytes for geometry column".to_string(),
            )),
        }
    }

    fn type_name() -> String {
        "WkbGeometry".to_string()
    }

    // If you want geometry arrays, define the array type. Otherwise use anything (Byte):
    fn array_type() -> ArrayType {
        ArrayType::Bytes
    }

    // Let’s define the reported column type for schema generation, etc.
    // Because it's a real PostGIS geometry column, we'll call it "Custom(\"geometry\")"
    fn column_type() -> ColumnType {
        ColumnType::Custom(SeaRc::new("geometry".to_string()))
    }
}
