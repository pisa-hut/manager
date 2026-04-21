use crate::entity::scenario_file;
use sea_orm::*;

pub async fn find_by_scenario(
    db: &DatabaseConnection,
    scenario_id: i32,
) -> Result<Vec<scenario_file::Model>, DbErr> {
    scenario_file::Entity::find()
        .filter(scenario_file::Column::ScenarioId.eq(scenario_id))
        .all(db)
        .await
}

pub async fn get(
    db: &DatabaseConnection,
    scenario_id: i32,
    relative_path: &str,
) -> Result<Option<scenario_file::Model>, DbErr> {
    scenario_file::Entity::find()
        .filter(scenario_file::Column::ScenarioId.eq(scenario_id))
        .filter(scenario_file::Column::RelativePath.eq(relative_path))
        .one(db)
        .await
}

pub async fn put(
    db: &DatabaseConnection,
    scenario_id: i32,
    relative_path: String,
    content: Vec<u8>,
    content_sha256: String,
) -> Result<scenario_file::Model, DbErr> {
    if let Some(existing) = get(db, scenario_id, &relative_path).await? {
        let mut am: scenario_file::ActiveModel = existing.into();
        am.content = Set(content);
        am.content_sha256 = Set(content_sha256);
        am.update(db).await
    } else {
        let am = scenario_file::ActiveModel {
            scenario_id: Set(scenario_id),
            relative_path: Set(relative_path),
            content: Set(content),
            content_sha256: Set(content_sha256),
            ..Default::default()
        };
        am.insert(db).await
    }
}

pub async fn delete(
    db: &DatabaseConnection,
    scenario_id: i32,
    relative_path: &str,
) -> Result<u64, DbErr> {
    let res = scenario_file::Entity::delete_many()
        .filter(scenario_file::Column::ScenarioId.eq(scenario_id))
        .filter(scenario_file::Column::RelativePath.eq(relative_path))
        .exec(db)
        .await?;
    Ok(res.rows_affected)
}
