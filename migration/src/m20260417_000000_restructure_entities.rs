use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Add sample_type to site_replicates (nullable for backfill)
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ADD COLUMN sample_type TEXT NULL"#,
        )
        .await?;

        // 2. Backfill from samples first (most common), then fall back to isolates.
        db.execute_unprepared(
            r#"UPDATE site_replicates sr SET sample_type = s.sample_type
               FROM samples s
               WHERE s.site_replicate_id = sr.id
               AND sr.sample_type IS NULL"#,
        )
        .await?;
        db.execute_unprepared(
            r#"UPDATE site_replicates sr SET sample_type = i.sample_type
               FROM isolates i
               WHERE i.site_replicate_id = sr.id
               AND sr.sample_type IS NULL"#,
        )
        .await?;
        // Any remaining replicates with no children default to Snow to pass the
        // NOT NULL constraint. Admin can fix after the fact.
        db.execute_unprepared(
            r#"UPDATE site_replicates SET sample_type = 'Snow' WHERE sample_type IS NULL"#,
        )
        .await?;

        // 3. Validate no NULLs remain, then lock down.
        let null_count = db
            .query_one(Statement::from_string(
                sea_orm_migration::sea_orm::DatabaseBackend::Postgres,
                r#"SELECT COUNT(*) as count FROM site_replicates WHERE sample_type IS NULL"#
                    .to_owned(),
            ))
            .await?;
        if let Some(row) = null_count {
            let count: i64 = row.try_get("", "count")?;
            if count > 0 {
                return Err(DbErr::Custom(format!(
                    "Found {count} site_replicates with NULL sample_type after backfill"
                )));
            }
        }
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ALTER COLUMN sample_type SET NOT NULL"#,
        )
        .await?;
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates ADD CONSTRAINT site_replicates_sample_type_check CHECK (sample_type IN ('Snow', 'Soil'))"#,
        )
        .await?;

        // 4. Drop sample_type from samples.
        db.execute_unprepared(
            r#"ALTER TABLE samples DROP CONSTRAINT samples_sample_type_check"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE samples DROP COLUMN sample_type"#)
            .await?;

        // 5. Drop sample_type from isolates.
        db.execute_unprepared(
            r#"ALTER TABLE isolates DROP CONSTRAINT isolates_sample_type_check"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE isolates DROP COLUMN sample_type"#)
            .await?;

        // 6. Add is_available to samples.
        db.execute_unprepared(
            r#"ALTER TABLE samples ADD COLUMN is_available BOOLEAN NOT NULL DEFAULT TRUE"#,
        )
        .await?;

        // 7. Add description to isolates.
        db.execute_unprepared(r#"ALTER TABLE isolates ADD COLUMN description TEXT NULL"#)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Reverse 7.
        db.execute_unprepared(r#"ALTER TABLE isolates DROP COLUMN description"#)
            .await?;

        // Reverse 6.
        db.execute_unprepared(r#"ALTER TABLE samples DROP COLUMN is_available"#)
            .await?;

        // Reverse 5: re-add sample_type to isolates, backfill from parent.
        db.execute_unprepared(r#"ALTER TABLE isolates ADD COLUMN sample_type TEXT NULL"#)
            .await?;
        db.execute_unprepared(
            r#"UPDATE isolates i SET sample_type = sr.sample_type
               FROM site_replicates sr
               WHERE i.site_replicate_id = sr.id"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE isolates ALTER COLUMN sample_type SET NOT NULL"#)
            .await?;
        db.execute_unprepared(
            r#"ALTER TABLE isolates ADD CONSTRAINT isolates_sample_type_check CHECK (sample_type IN ('Snow', 'Soil'))"#,
        )
        .await?;

        // Reverse 4: re-add sample_type to samples, backfill from parent.
        db.execute_unprepared(r#"ALTER TABLE samples ADD COLUMN sample_type TEXT NULL"#)
            .await?;
        db.execute_unprepared(
            r#"UPDATE samples s SET sample_type = sr.sample_type
               FROM site_replicates sr
               WHERE s.site_replicate_id = sr.id"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE samples ALTER COLUMN sample_type SET NOT NULL"#)
            .await?;
        db.execute_unprepared(
            r#"ALTER TABLE samples ADD CONSTRAINT samples_sample_type_check CHECK (sample_type IN ('Snow', 'Soil'))"#,
        )
        .await?;

        // Reverse 3 & 1: drop sample_type from site_replicates.
        db.execute_unprepared(
            r#"ALTER TABLE site_replicates DROP CONSTRAINT site_replicates_sample_type_check"#,
        )
        .await?;
        db.execute_unprepared(r#"ALTER TABLE site_replicates DROP COLUMN sample_type"#)
            .await?;

        Ok(())
    }
}
