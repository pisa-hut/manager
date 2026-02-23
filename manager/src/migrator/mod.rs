// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260224_103310_new_db_schema;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260224_103310_new_db_schema::Migration)]
    }
}
