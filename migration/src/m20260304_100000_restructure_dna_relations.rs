use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(r#"ALTER TABLE isolates DROP COLUMN dna_id"#).await?;
        db.execute_unprepared(r#"ALTER TABLE samples DROP COLUMN dna_id"#).await?;
        db.execute_unprepared(r#"TRUNCATE TABLE dna"#).await?;
        db.execute_unprepared(
            r#"ALTER TABLE dna ADD COLUMN site_replicate_id UUID NOT NULL REFERENCES site_replicates(id)"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE isolates ADD COLUMN genome_url TEXT NULL"#).await?;
        db.execute_unprepared(r#"ALTER TABLE site_replicates ADD COLUMN metagenome_url TEXT NULL"#)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(r#"ALTER TABLE site_replicates DROP COLUMN metagenome_url"#).await?;
        db.execute_unprepared(r#"ALTER TABLE isolates DROP COLUMN genome_url"#).await?;
        db.execute_unprepared(r#"ALTER TABLE dna DROP COLUMN site_replicate_id"#).await?;
        db.execute_unprepared(r#"ALTER TABLE samples ADD COLUMN dna_id UUID NULL REFERENCES dna(id)"#)
            .await?;
        db.execute_unprepared(r#"ALTER TABLE isolates ADD COLUMN dna_id UUID NULL REFERENCES dna(id)"#)
            .await?;

        Ok(())
    }
}
