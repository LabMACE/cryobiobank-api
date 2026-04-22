use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let add_created_at = r#"
            ALTER TABLE areas ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            ALTER TABLE sites ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            ALTER TABLE site_replicates ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            ALTER TABLE samples ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            ALTER TABLE isolates ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
            ALTER TABLE dna ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
        "#;

        db.execute_unprepared(add_created_at).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let drop_created_at = r#"
            ALTER TABLE areas DROP COLUMN IF EXISTS created_at;
            ALTER TABLE sites DROP COLUMN IF EXISTS created_at;
            ALTER TABLE site_replicates DROP COLUMN IF EXISTS created_at;
            ALTER TABLE samples DROP COLUMN IF EXISTS created_at;
            ALTER TABLE isolates DROP COLUMN IF EXISTS created_at;
            ALTER TABLE dna DROP COLUMN IF EXISTS created_at;
        "#;

        db.execute_unprepared(drop_created_at).await?;
        Ok(())
    }
}
