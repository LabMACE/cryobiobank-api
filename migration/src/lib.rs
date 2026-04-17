pub use sea_orm_migration::prelude::*;

mod m20250109_143445_create_initial_tables;
mod m20250122_142104_avoid_postgis_geometry;
mod m20250124_113623_set_isolates_photo_to_text;
mod m20250206_134125_add_areas;
mod m20250926_140000_add_private_fields;
mod m20260219_000000_add_sample_type_enum;
mod m20260304_000000_move_sample_type_to_samples;
mod m20260304_100000_restructure_dna_relations;
mod m20260327_000000_schema_cleanup;
mod m20260417_000000_restructure_entities;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250109_143445_create_initial_tables::Migration),
            Box::new(m20250122_142104_avoid_postgis_geometry::Migration),
            Box::new(m20250124_113623_set_isolates_photo_to_text::Migration),
            Box::new(m20250206_134125_add_areas::Migration),
            Box::new(m20250926_140000_add_private_fields::Migration),
            Box::new(m20260219_000000_add_sample_type_enum::Migration),
            Box::new(m20260304_000000_move_sample_type_to_samples::Migration),
            Box::new(m20260304_100000_restructure_dna_relations::Migration),
            Box::new(m20260327_000000_schema_cleanup::Migration),
            Box::new(m20260417_000000_restructure_entities::Migration),
        ]
    }
}
