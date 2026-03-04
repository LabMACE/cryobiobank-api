use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Add sample_type column to samples (temporarily nullable)
        db.execute_unprepared(
            r#"ALTER TABLE samples ADD COLUMN sample_type TEXT NULL"#,
        )
        .await?;

        // 2. Backfill samples from type_of_sample using pattern matching
        db.execute_unprepared(
            r#"UPDATE samples SET sample_type = 'Snow' WHERE LOWER(type_of_sample) LIKE '%snow%'"#,
        )
        .await?;
        db.execute_unprepared(
            r#"UPDATE samples SET sample_type = 'Soil' WHERE LOWER(type_of_sample) LIKE '%soil%'"#,
        )
        .await?;

        // 3. Backfill remaining samples from parent site_replicate
        db.execute_unprepared(
            r#"UPDATE samples SET sample_type = sr.sample_type
               FROM site_replicates sr
               WHERE samples.site_replicate_id = sr.id
               AND samples.sample_type IS NULL"#,
        )
        .await?;

        // 4. Backfill isolates from parent site_replicate (for null sample_type)
        db.execute_unprepared(
            r#"UPDATE isolates SET sample_type = sr.sample_type
               FROM site_replicates sr
               WHERE isolates.site_replicate_id = sr.id
               AND isolates.sample_type IS NULL"#,
        )
        .await?;

        // 5. Validate no NULLs remain in samples.sample_type
        let null_samples = db
            .query_one(Statement::from_string(
                sea_orm_migration::sea_orm::DatabaseBackend::Postgres,
                r#"SELECT COUNT(*) as count FROM samples WHERE sample_type IS NULL"#.to_owned(),
            ))
            .await?;
        if let Some(row) = null_samples {
            let count: i64 = row.try_get("", "count")?;
            if count > 0 {
                return Err(DbErr::Custom(format!(
                    "Found {} samples with NULL sample_type after backfill. Please fix manually.",
                    count
                )));
            }
        }

        // Validate no NULLs remain in isolates.sample_type
        let null_isolates = db
            .query_one(Statement::from_string(
                sea_orm_migration::sea_orm::DatabaseBackend::Postgres,
                r#"SELECT COUNT(*) as count FROM isolates WHERE sample_type IS NULL"#.to_owned(),
            ))
            .await?;
        if let Some(row) = null_isolates {
            let count: i64 = row.try_get("", "count")?;
            if count > 0 {
                return Err(DbErr::Custom(format!(
                    "Found {} isolates with NULL sample_type after backfill. Please fix manually.",
                    count
                )));
            }
        }

        // 6. Set NOT NULL on both columns
        db.execute_unprepared(
            r#"ALTER TABLE samples ALTER COLUMN sample_type SET NOT NULL"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE isolates ALTER COLUMN sample_type SET NOT NULL"#,
        )
        .await?;

        // 7. Drop type_of_sample from samples
        db.execute_unprepared(
            r#"ALTER TABLE samples DROP COLUMN type_of_sample"#,
        )
        .await?;

        // 8. Drop sample_type from site_replicates
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates DROP COLUMN sample_type"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Restore sample_type on site_replicates
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ADD COLUMN sample_type TEXT NOT NULL DEFAULT 'Snow'"#,
        )
        .await?;

        // Restore type_of_sample on samples
        db.execute_unprepared(
            r#"ALTER TABLE samples ADD COLUMN type_of_sample TEXT NULL"#,
        )
        .await?;

        // Make isolates.sample_type nullable again
        db.execute_unprepared(
            r#"ALTER TABLE isolates ALTER COLUMN sample_type DROP NOT NULL"#,
        )
        .await?;

        // Make samples.sample_type nullable and then drop it
        db.execute_unprepared(
            r#"ALTER TABLE samples DROP COLUMN sample_type"#,
        )
        .await?;

        Ok(())
    }
}
