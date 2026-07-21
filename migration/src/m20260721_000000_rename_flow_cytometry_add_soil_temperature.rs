use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let statements = r#"
            ALTER TABLE field_records RENAME COLUMN bacterial_abundance TO flow_cytometry_cell_number;
            ALTER TABLE field_records ADD COLUMN soil_temperature_celsius DOUBLE PRECISION;
        "#;

        db.execute_unprepared(statements).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let statements = r#"
            ALTER TABLE field_records DROP COLUMN IF EXISTS soil_temperature_celsius;
            ALTER TABLE field_records RENAME COLUMN flow_cytometry_cell_number TO bacterial_abundance;
        "#;

        db.execute_unprepared(statements).await?;
        Ok(())
    }
}
