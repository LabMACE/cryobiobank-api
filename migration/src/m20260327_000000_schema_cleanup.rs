use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. samples.site_replicate_id: nullable → NOT NULL
        db.execute_unprepared(
            r#"ALTER TABLE samples ALTER COLUMN site_replicate_id SET NOT NULL"#,
        )
        .await?;

        // 2. areas.name: nullable → NOT NULL
        db.execute_unprepared(r#"ALTER TABLE areas ALTER COLUMN name SET NOT NULL"#)
            .await?;

        // 3. areas.colour: nullable → NOT NULL
        db.execute_unprepared(r#"ALTER TABLE areas ALTER COLUMN colour SET NOT NULL"#)
            .await?;

        // 4. areas.name: add UNIQUE constraint (Rust model declares it but DB was missing it)
        db.execute_unprepared(
            r#"ALTER TABLE areas ADD CONSTRAINT areas_name_key UNIQUE (name)"#,
        )
        .await?;

        // 5. Drop old sequencing/publication columns from site_replicates
        //    (sequencing replaced by metagenome_url, publications unused)
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates DROP COLUMN sequencing_results_16s"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates DROP COLUMN sequencing_metagenomics"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates DROP COLUMN relevant_publications"#,
        )
        .await?;

        // 6. Add CHECK constraints for sample_type enum values
        db.execute_unprepared(
            r#"ALTER TABLE isolates ADD CONSTRAINT isolates_sample_type_check CHECK (sample_type IN ('Snow', 'Soil'))"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE samples ADD CONSTRAINT samples_sample_type_check CHECK (sample_type IN ('Snow', 'Soil'))"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(r#"ALTER TABLE samples DROP CONSTRAINT samples_sample_type_check"#)
            .await?;
        db.execute_unprepared(
            r#"ALTER TABLE isolates DROP CONSTRAINT isolates_sample_type_check"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ADD COLUMN relevant_publications TEXT NULL"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ADD COLUMN sequencing_metagenomics TEXT NULL"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ADD COLUMN sequencing_results_16s TEXT NULL"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE areas DROP CONSTRAINT areas_name_key"#)
            .await?;
        db.execute_unprepared(r#"ALTER TABLE areas ALTER COLUMN colour DROP NOT NULL"#)
            .await?;
        db.execute_unprepared(r#"ALTER TABLE areas ALTER COLUMN name DROP NOT NULL"#)
            .await?;
        db.execute_unprepared(
            r#"ALTER TABLE samples ALTER COLUMN site_replicate_id DROP NOT NULL"#,
        )
        .await?;

        Ok(())
    }
}
