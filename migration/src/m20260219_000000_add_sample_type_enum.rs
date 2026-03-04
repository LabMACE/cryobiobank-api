use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Check for invalid sample_type values in site_replicates
        let invalid_count = db
            .query_one(Statement::from_string(
                sea_orm_migration::sea_orm::DatabaseBackend::Postgres,
                r#"SELECT COUNT(*) as count FROM site_replicates WHERE LOWER(sample_type) NOT IN ('snow', 'soil')"#.to_owned(),
            ))
            .await?;

        if let Some(row) = invalid_count {
            let count: i64 = row.try_get("", "count")?;
            if count > 0 {
                return Err(DbErr::Custom(format!(
                    "Found {} site_replicates with sample_type not in ('Snow', 'Soil'). Please fix these before migrating.",
                    count
                )));
            }
        }

        // Normalize existing values to title case
        let normalize = r#"
            UPDATE site_replicates SET sample_type = 'Snow' WHERE LOWER(sample_type) = 'snow';
            UPDATE site_replicates SET sample_type = 'Soil' WHERE LOWER(sample_type) = 'soil';
        "#;
        db.execute_unprepared(normalize).await?;

        // Add sample_type column to isolates
        let add_column = r#"
            ALTER TABLE isolates ADD COLUMN sample_type TEXT NULL;
        "#;
        db.execute_unprepared(add_column).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let drop_column = r#"
            ALTER TABLE isolates DROP COLUMN IF EXISTS sample_type;
        "#;
        db.execute_unprepared(drop_column).await?;

        Ok(())
    }
}
