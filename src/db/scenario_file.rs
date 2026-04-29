use crate::entity::scenario_file;
use sea_orm::sea_query::OnConflict;
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

/// Upsert a scenario file's content. Single round-trip via
/// `INSERT ... ON CONFLICT (scenario_id, relative_path) DO UPDATE`,
/// which also closes the get-then-insert race that would have let two
/// concurrent uploads of the same file produce a constraint violation
/// or last-write-wins surprises.
pub async fn put(
    db: &DatabaseConnection,
    scenario_id: i32,
    relative_path: String,
    content: Vec<u8>,
    content_sha256: String,
) -> Result<scenario_file::Model, DbErr> {
    let am = scenario_file::ActiveModel {
        scenario_id: Set(scenario_id),
        relative_path: Set(relative_path),
        content: Set(content),
        content_sha256: Set(content_sha256),
        ..Default::default()
    };

    scenario_file::Entity::insert(am)
        .on_conflict(
            OnConflict::columns([
                scenario_file::Column::ScenarioId,
                scenario_file::Column::RelativePath,
            ])
            .update_columns([
                scenario_file::Column::Content,
                scenario_file::Column::ContentSha256,
            ])
            .to_owned(),
        )
        .exec_with_returning(db)
        .await
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
