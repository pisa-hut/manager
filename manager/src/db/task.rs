use crate::entity::task;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<task::Model>, DbErr> {
    task::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    plan_id: i32,
    av_id: i32,
) -> Result<task::Model, DbErr> {
    let active = task::ActiveModel {
        plan_id: Set(plan_id),
        av_id: Set(av_id),
        status: Set(Some("created".to_string())),
        ..Default::default()
    };

    active.insert(db).await
}
