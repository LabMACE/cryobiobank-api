pub use sea_orm_migration::prelude::*;

mod m20250109_143445_create_initial_tables;
mod m20250122_142104_avoid_postgis_geometry;
mod m20250124_113623_set_isolates_photo_to_text;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250109_143445_create_initial_tables::Migration),
            Box::new(m20250122_142104_avoid_postgis_geometry::Migration),
            Box::new(m20250124_113623_set_isolates_photo_to_text::Migration),
        ]
    }
}
