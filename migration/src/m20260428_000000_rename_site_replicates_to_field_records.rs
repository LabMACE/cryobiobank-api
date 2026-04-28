use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let rename = r#"
            ALTER TABLE site_replicates RENAME TO field_records;
            ALTER TABLE samples RENAME COLUMN site_replicate_id TO field_record_id;
            ALTER TABLE isolates RENAME COLUMN site_replicate_id TO field_record_id;
            ALTER TABLE dna RENAME COLUMN site_replicate_id TO field_record_id;

            ALTER TABLE samples RENAME CONSTRAINT fk_sample_site_replicate_id TO fk_sample_field_record_id;
            ALTER TABLE isolates RENAME CONSTRAINT fk_isolate_site_replicate_id TO fk_isolate_field_record_id;
            ALTER TABLE field_records RENAME CONSTRAINT site_replicates_sample_type_check TO field_records_sample_type_check;
        "#;

        db.execute_unprepared(rename).await?;

        // DNA FK was auto-named by Postgres — rename if it exists
        let rename_dna_fk = r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1 FROM information_schema.table_constraints
                    WHERE constraint_name = 'dna_site_replicate_id_fkey'
                      AND table_name = 'dna'
                ) THEN
                    ALTER TABLE dna RENAME CONSTRAINT dna_site_replicate_id_fkey TO dna_field_record_id_fkey;
                END IF;
            END $$;
        "#;

        db.execute_unprepared(rename_dna_fk).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let revert = r#"
            ALTER TABLE field_records RENAME TO site_replicates;
            ALTER TABLE samples RENAME COLUMN field_record_id TO site_replicate_id;
            ALTER TABLE isolates RENAME COLUMN field_record_id TO site_replicate_id;
            ALTER TABLE dna RENAME COLUMN field_record_id TO site_replicate_id;

            ALTER TABLE samples RENAME CONSTRAINT fk_sample_field_record_id TO fk_sample_site_replicate_id;
            ALTER TABLE isolates RENAME CONSTRAINT fk_isolate_field_record_id TO fk_isolate_site_replicate_id;
            ALTER TABLE site_replicates RENAME CONSTRAINT field_records_sample_type_check TO site_replicates_sample_type_check;
        "#;

        db.execute_unprepared(revert).await?;

        let revert_dna_fk = r#"
            DO $$
            BEGIN
                IF EXISTS (
                    SELECT 1 FROM information_schema.table_constraints
                    WHERE constraint_name = 'dna_field_record_id_fkey'
                      AND table_name = 'dna'
                ) THEN
                    ALTER TABLE dna RENAME CONSTRAINT dna_field_record_id_fkey TO dna_site_replicate_id_fkey;
                END IF;
            END $$;
        "#;

        db.execute_unprepared(revert_dna_fk).await?;
        Ok(())
    }
}
