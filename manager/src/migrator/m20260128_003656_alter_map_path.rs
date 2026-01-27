use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260128_003656_alter_map_path"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add new path columns
        manager
            .alter_table(
                Table::alter()
                    .table(Map::Table)
                    .add_column(ColumnDef::new(Map::XodrPath).string().null())
                    .add_column(ColumnDef::new(Map::OsmPath).string().null())
                    .to_owned(),
            )
            .await?;

        // 2. Migrate data from old schema
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE map
                SET
                  xodr_path = CASE WHEN xodr THEN path ELSE NULL END,
                  osm_path  = CASE WHEN osm  THEN path ELSE NULL END
                "#,
            )
            .await?;

        // 3. Drop old columns
        manager
            .alter_table(
                Table::alter()
                    .table(Map::Table)
                    .drop_column(Map::Path)
                    .drop_column(Map::Xodr)
                    .drop_column(Map::Osm)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Re-add old columns
        manager
            .alter_table(
                Table::alter()
                    .table(Map::Table)
                    .add_column(ColumnDef::new(Map::Path).string().null())
                    .add_column(
                        ColumnDef::new(Map::Xodr)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(Map::Osm).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        // 2. Reverse data migration
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE map
                SET
                  path = COALESCE(xodr_path, osm_path),
                  xodr = xodr_path IS NOT NULL,
                  osm  = osm_path  IS NOT NULL
                "#,
            )
            .await?;

        // 3. Drop new columns
        manager
            .alter_table(
                Table::alter()
                    .table(Map::Table)
                    .drop_column(Map::XodrPath)
                    .drop_column(Map::OsmPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
#[derive(Iden)]
enum Map {
    Table,
    Path,
    Xodr,
    Osm,
    XodrPath,
    OsmPath,
}
