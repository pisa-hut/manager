use crate::entity::monitor;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<monitor::Model>, DbErr> {
    monitor::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    module_path: String,
) -> Result<monitor::Model, DbErr> {
    let active = monitor::ActiveModel {
        name: Set(name),
        module_path: Set(module_path),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn monitor_exists(db: &DatabaseConnection, monitor_id: i32) -> Result<bool, DbErr> {
    let count = monitor::Entity::find_by_id(monitor_id)
        .one(db)
        .await?
        .is_some() as i64;

    Ok(count > 0)
}

pub async fn get_by_id(
    db: &DatabaseConnection,
    monitor_id: i32,
) -> Result<Option<monitor::Model>, DbErr> {
    monitor::Entity::find_by_id(monitor_id).one(db).await
}

// `set_config` and `clear_config` are now provided generically via the
// `db::ConfigBearing` trait impl on `monitor::Model`.
