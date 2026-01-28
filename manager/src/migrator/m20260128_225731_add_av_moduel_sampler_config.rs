use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260128_225731_add_av_moduel_sampler_config"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Av::Table)
                    .add_column(ColumnDef::new(Av::ModulePath).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .add_column(ColumnDef::new(Sampler::ConfigPath).string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Av::Table)
                    .drop_column(Av::ModulePath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Sampler::Table)
                    .drop_column(Sampler::ConfigPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Av {
    Table,
    ModulePath,
}

#[derive(Iden)]
enum Sampler {
    Table,
    ConfigPath,
}
