use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let add_private_fields = r#"
            ALTER TABLE sites ADD COLUMN is_private BOOLEAN DEFAULT FALSE;
            ALTER TABLE isolates ADD COLUMN is_private BOOLEAN DEFAULT FALSE;
            ALTER TABLE samples ADD COLUMN is_private BOOLEAN DEFAULT FALSE;
            ALTER TABLE dna ADD COLUMN is_private BOOLEAN DEFAULT FALSE;
            ALTER TABLE site_replicates ADD COLUMN is_private BOOLEAN DEFAULT FALSE;
            ALTER TABLE areas ADD COLUMN is_private BOOLEAN DEFAULT FALSE;
        "#;

        db.execute_unprepared(add_private_fields).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let drop_private_fields = r#"
            ALTER TABLE sites DROP COLUMN IF EXISTS is_private;
            ALTER TABLE isolates DROP COLUMN IF EXISTS is_private;
            ALTER TABLE samples DROP COLUMN IF EXISTS is_private;
            ALTER TABLE dna DROP COLUMN IF EXISTS is_private;
            ALTER TABLE site_replicates DROP COLUMN IF EXISTS is_private;
            ALTER TABLE areas DROP COLUMN IF EXISTS is_private;
        "#;

        db.execute_unprepared(drop_private_fields).await?;
        Ok(())
    }
}
