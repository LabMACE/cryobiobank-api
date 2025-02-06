use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create the area table
        let create_area_table = r#"
            CREATE TABLE IF NOT EXISTS areas (
                id UUID NOT NULL,
                name TEXT,
                description TEXT,
                colour TEXT,
                PRIMARY KEY (id)
            );
        "#;
        db.execute_unprepared(create_area_table).await?;

        // Add the area_id column to sites
        let add_area_id_column = r#"
            ALTER TABLE sites
            ADD COLUMN area_id UUID;
        "#;
        db.execute_unprepared(add_area_id_column).await?;

        // Add foreign key constraint linking sites.area_id to area.id
        let add_foreign_key = r#"
            ALTER TABLE sites
            ADD CONSTRAINT fk_sites_area_id
            FOREIGN KEY (area_id) REFERENCES areas(id)
            ON DELETE SET NULL
            ON UPDATE CASCADE;
        "#;
        db.execute_unprepared(add_foreign_key).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove the foreign key constraint
        let drop_foreign_key = r#"
            ALTER TABLE sites
            DROP CONSTRAINT IF EXISTS fk_sites_area_id;
        "#;
        db.execute_unprepared(drop_foreign_key).await?;

        // Drop the area_id column from sites
        let drop_area_id_column = r#"
            ALTER TABLE sites
            DROP COLUMN IF EXISTS area_id;
        "#;
        db.execute_unprepared(drop_area_id_column).await?;

        // Drop the area table
        let drop_area_table = r#"
            DROP TABLE IF EXISTS area;
        "#;
        db.execute_unprepared(drop_area_table).await?;

        Ok(())
    }
}
