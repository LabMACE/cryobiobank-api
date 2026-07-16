use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let add_columns = r#"
            ALTER TABLE field_records ADD COLUMN treatment TEXT;
            ALTER TABLE field_records ADD COLUMN campaign TEXT;
            ALTER TABLE field_records ADD COLUMN water_content DOUBLE PRECISION;
            ALTER TABLE field_records ADD COLUMN total_carbon DOUBLE PRECISION;
            ALTER TABLE field_records ADD COLUMN total_organic_carbon DOUBLE PRECISION;
            ALTER TABLE field_records ADD COLUMN total_nitrogen DOUBLE PRECISION;
            ALTER TABLE dna ADD COLUMN volume DOUBLE PRECISION;
            ALTER TABLE dna ADD COLUMN concentration DOUBLE PRECISION;
        "#;

        db.execute_unprepared(add_columns).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let drop_columns = r#"
            ALTER TABLE field_records DROP COLUMN IF EXISTS treatment;
            ALTER TABLE field_records DROP COLUMN IF EXISTS campaign;
            ALTER TABLE field_records DROP COLUMN IF EXISTS water_content;
            ALTER TABLE field_records DROP COLUMN IF EXISTS total_carbon;
            ALTER TABLE field_records DROP COLUMN IF EXISTS total_organic_carbon;
            ALTER TABLE field_records DROP COLUMN IF EXISTS total_nitrogen;
            ALTER TABLE dna DROP COLUMN IF EXISTS volume;
            ALTER TABLE dna DROP COLUMN IF EXISTS concentration;
        "#;

        db.execute_unprepared(drop_columns).await?;
        Ok(())
    }
}
