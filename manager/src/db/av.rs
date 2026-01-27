use crate::entity::av;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<av::Model>, DbErr> {
    av::Entity::find().all(db).await
}

pub async fn create(db: &DatabaseConnection, active: av::ActiveModel) -> Result<av::Model, DbErr> {
    active.insert(db).await
}
