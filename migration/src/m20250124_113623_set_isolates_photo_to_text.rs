use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Isolates::Table)
                    .modify_column(ColumnDef::new(Isolates::Photo).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Isolates::Table)
                    .modify_column(ColumnDef::new(Isolates::Photo).binary().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Isolates {
    Table,
    Photo,
}
