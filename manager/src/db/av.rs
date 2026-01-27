use crate::{entity::av, http::dto::av::CreateAvRequest};
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<av::Model>, DbErr> {
    av::Entity::find().all(db).await
}

pub async fn create(db: &DatabaseConnection, req: CreateAvRequest) -> Result<av::Model, DbErr> {
    let active = av::ActiveModel {
        name: Set(req.name),
        ..Default::default()
    };

    active.insert(db).await
}
