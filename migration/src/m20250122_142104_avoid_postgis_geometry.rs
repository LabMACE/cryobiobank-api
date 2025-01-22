use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Step 1: Add new columns for latitude, longitude, and elevation.
        let sql_add_columns = r#"
            ALTER TABLE sites
            ADD COLUMN latitude_4326 double precision,
            ADD COLUMN longitude_4326 double precision,
            ADD COLUMN elevation_metres double precision;
        "#;
        conn.execute_unprepared(sql_add_columns).await?;

        // Step 2: Populate the new columns using the geometry column.
        let sql_populate_columns = r#"
            UPDATE sites
            SET latitude_4326 = ST_Y(geometry),
                longitude_4326 = ST_X(geometry),
                elevation_metres = ST_Z(geometry);
        "#;
        conn.execute_unprepared(sql_populate_columns).await?;

        // Step 3: Drop the geometry column.
        let sql_drop_geometry = r#"
            ALTER TABLE sites
            DROP COLUMN geometry;
        "#;
        conn.execute_unprepared(sql_drop_geometry).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Step 1: Re-add the geometry column with the appropriate PostGIS type.
        let sql_add_geometry = r#"
            ALTER TABLE sites
            ADD COLUMN geometry geometry(POINTZ,4326) NOT NULL;
        "#;
        conn.execute_unprepared(sql_add_geometry).await?;

        // Step 2: Populate the geometry column using the new coordinate columns.
        let sql_populate_geometry = r#"
            UPDATE sites
            SET geometry = ST_SetSRID(ST_MakePoint(longitude_4326, latitude_4326, elevation_metres), 4326);
        "#;
        conn.execute_unprepared(sql_populate_geometry).await?;

        // Step 3: Drop the new columns.
        let sql_drop_new_columns = r#"
            ALTER TABLE sites
            DROP COLUMN latitude_4326,
            DROP COLUMN longitude_4326,
            DROP COLUMN elevation_metres;
        "#;
        conn.execute_unprepared(sql_drop_new_columns).await?;

        Ok(())
    }
}
