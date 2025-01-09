use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // PostGIS specific
        let create_all_tables = r#"
            CREATE EXTENSION IF NOT EXISTS postgis;

            CREATE TABLE IF NOT EXISTS dna (
              id                UUID NOT NULL,
              name              TEXT NOT NULL,
              description       TEXT,
              extraction_method TEXT,
              PRIMARY KEY (id),
              UNIQUE (name)
            );

            CREATE TABLE IF NOT EXISTS sites (
              id        UUID NOT NULL,
              name      TEXT NOT NULL,
              -- 3D geometry point (lat, lon, elevation) with SRID=4326
              geometry  geometry(POINTZ, 4326) NOT NULL,
              PRIMARY KEY (id),
              UNIQUE (name)
            );

            CREATE TABLE IF NOT EXISTS site_samples (
              id                                 UUID NOT NULL,
              name                               TEXT NULL,
              sample_type                        TEXT NOT NULL,
              sampling_date                      DATE NOT NULL,
              sample_depth_cm                    INT NULL,
              snow_depth                         INT NULL,
              air_temperature_celsius            DOUBLE PRECISION NULL,
              snow_temperature_celsius           DOUBLE PRECISION NULL,
              photosynthetic_active_radiation    INT NULL,
              bacterial_abundance                BIGINT NULL,
              cfu_count_r2a                      INT NULL,
              cfu_count_another                  INT NULL,
              ph                                 DOUBLE PRECISION NULL,
              ions_fluoride                      DOUBLE PRECISION NULL,
              ions_chloride                      DOUBLE PRECISION NULL,
              ions_nitrite                       DOUBLE PRECISION NULL,
              ions_nitrate                       DOUBLE PRECISION NULL,
              ions_bromide                       DOUBLE PRECISION NULL,
              ions_sulfate                       DOUBLE PRECISION NULL,
              ions_phosphate                     DOUBLE PRECISION NULL,
              ions_sodium                        DOUBLE PRECISION NULL,
              ions_ammonium                      DOUBLE PRECISION NULL,
              ions_potassium                     DOUBLE PRECISION NULL,
              ions_magnesium                     DOUBLE PRECISION NULL,
              ions_calcium                       DOUBLE PRECISION NULL,
              organic_acids_formate              DOUBLE PRECISION NULL,
              organic_acids_malate               DOUBLE PRECISION NULL,
              organic_acids_propionate           DOUBLE PRECISION NULL,
              organic_acids_citrate              DOUBLE PRECISION NULL,
              organic_acids_lactate              DOUBLE PRECISION NULL,
              organic_acids_butyrate             DOUBLE PRECISION NULL,
              organic_acids_oxalate              DOUBLE PRECISION NULL,
              organic_acids_acetate              DOUBLE PRECISION NULL,
              sequencing_results_16s             TEXT NULL,
              sequencing_metagenomics            TEXT NULL,
              relevant_publications              TEXT NULL,
              PRIMARY KEY (id),
              UNIQUE (name)
            );

            CREATE TABLE IF NOT EXISTS isolates (
              id                       UUID NOT NULL,
              name                     TEXT NOT NULL,
              taxonomy                 TEXT,
              photo                    BYTEA,
              site_id                  UUID,
              temperature_of_isolation DOUBLE PRECISION,
              media_used_for_isolation TEXT,
              storage_location         TEXT,
              dna_id                   UUID,
              PRIMARY KEY (id),
              UNIQUE (name),
              CONSTRAINT fk_isolate_site_id
                FOREIGN KEY (site_id) REFERENCES sites(id)
                ON DELETE SET NULL
                ON UPDATE CASCADE,
              CONSTRAINT fk_isolate_dna_id
                FOREIGN KEY (dna_id) REFERENCES dna(id)
                ON DELETE SET NULL
                ON UPDATE CASCADE
            );

            CREATE TABLE IF NOT EXISTS samples (
              id               UUID NOT NULL,
              name             TEXT NOT NULL,
              site_id          UUID,
              type_of_sample   TEXT,
              storage_location TEXT,
              description      TEXT,
              dna_id           UUID,
              PRIMARY KEY (id),
              UNIQUE (name),
              CONSTRAINT fk_sample_site_id
                FOREIGN KEY (site_id) REFERENCES sites(id)
                ON DELETE SET NULL
                ON UPDATE CASCADE,
              CONSTRAINT fk_sample_dna_id
                FOREIGN KEY (dna_id) REFERENCES dna(id)
                ON DELETE SET NULL
                ON UPDATE CASCADE
            );
        "#;

        db.execute_unprepared(create_all_tables).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop tables in reverse order of creation, so references are removed cleanly
        let drop_all = r#"
            DROP TABLE IF EXISTS samples;
            DROP TABLE IF EXISTS isolates;
            DROP TABLE IF EXISTS site_samples;
            DROP TABLE IF EXISTS sites;
            DROP TABLE IF EXISTS dna;
        "#;

        db.execute_unprepared(drop_all).await?;
        Ok(())
    }
}
