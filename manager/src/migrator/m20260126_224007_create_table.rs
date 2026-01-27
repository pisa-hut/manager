// src/migrator/m20220602_000001_create_bakery_table.rs (create new file)

use sea_orm_migration::prelude::*;

use crate::migrator::sea_orm::EnumIter;
use sea_orm::Iterable;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260126_224007_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Worker::Table)
                    .col(
                        ColumnDef::new(Worker::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Worker::JobId).integer().not_null())
                    .col(ColumnDef::new(Worker::NodeList).string().not_null())
                    .col(ColumnDef::new(Worker::Hostname).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(AV::Table)
                    .col(
                        ColumnDef::new(AV::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AV::Name).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Scenario::Table)
                    .col(
                        ColumnDef::new(Scenario::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Scenario::Title).string().not_null())
                    .col(ColumnDef::new(Scenario::Description).string())
                    .col(ColumnDef::new(Scenario::Path).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Map::Table)
                    .col(
                        ColumnDef::new(Map::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Map::Name).string().not_null())
                    .col(ColumnDef::new(Map::XODR).boolean().not_null())
                    .col(ColumnDef::new(Map::OSM).boolean().not_null())
                    .col(ColumnDef::new(Map::Path).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Plan::Table)
                    .col(
                        ColumnDef::new(Plan::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Plan::Name).string().not_null())
                    .col(ColumnDef::new(Plan::MapId).integer().not_null())
                    .col(ColumnDef::new(Plan::ScenarioId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Plan::Table, Plan::MapId)
                            .to(Map::Table, Map::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Plan::Table, Plan::ScenarioId)
                            .to(Scenario::Table, Scenario::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .col(
                        ColumnDef::new(Task::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Task::PlanId).integer().not_null())
                    .col(ColumnDef::new(Task::AvId).integer().not_null())
                    .col(ColumnDef::new(Task::WorkerId).integer().null())
                    .col(
                        ColumnDef::new(Task::Status)
                            .enumeration(Alias::new("task_status"), TaskStatus::iter()),
                    )
                    .col(
                        ColumnDef::new(Task::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Task::ExecutedAt).timestamp())
                    .col(ColumnDef::new(Task::FinishedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::PlanId)
                            .to(Plan::Table, Plan::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::AvId)
                            .to(AV::Table, AV::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::WorkerId)
                            .to(Worker::Table, Worker::Id),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Worker::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AV::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scenario::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Map::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Plan::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Worker {
    Table,
    Id,
    JobId,
    NodeList,
    Hostname,
}

#[derive(DeriveIden)]
enum AV {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum Scenario {
    Table,
    Id,
    Title,
    Description,
    Path,
}

#[derive(DeriveIden)]
enum Map {
    Table,
    Id,
    Name,
    XODR,
    OSM,
    Path,
}

#[derive(DeriveIden)]
enum Plan {
    Table,
    Id,
    Name,
    MapId,
    ScenarioId,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    PlanId,
    AvId,
    WorkerId,
    Status,
    CreatedAt,
    ExecutedAt,
    FinishedAt,
}

#[derive(Iden, EnumIter)]
enum TaskStatus {
    #[iden = "pending"]
    Pending,
    #[iden = "in_progress"]
    InProgress,
    #[iden = "completed"]
    Completed,
    #[iden = "failed"]
    Failed,
}
