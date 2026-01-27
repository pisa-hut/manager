// src/migrator/mod.rs (create new file)

use sea_orm_migration::prelude::*;

mod m20260127_225907_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260127_225907_create_table::Migration)]
    }
}
