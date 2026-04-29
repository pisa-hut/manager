use crate::entity::map_file;
use sea_orm::sea_query::OnConflict;
use sea_orm::*;

pub async fn find_by_map(
    db: &DatabaseConnection,
    map_id: i32,
) -> Result<Vec<map_file::Model>, DbErr> {
    map_file::Entity::find()
        .filter(map_file::Column::MapId.eq(map_id))
        .all(db)
        .await
}

pub async fn get(
    db: &DatabaseConnection,
    map_id: i32,
    relative_path: &str,
) -> Result<Option<map_file::Model>, DbErr> {
    map_file::Entity::find()
        .filter(map_file::Column::MapId.eq(map_id))
        .filter(map_file::Column::RelativePath.eq(relative_path))
        .one(db)
        .await
}

/// Upsert a map file's content. Single round-trip via
/// `INSERT ... ON CONFLICT (map_id, relative_path) DO UPDATE`, which
/// also closes the get-then-insert race that would have let two
/// concurrent uploads of the same file produce a constraint violation
/// or last-write-wins surprises.
pub async fn put(
    db: &DatabaseConnection,
    map_id: i32,
    relative_path: String,
    content: Vec<u8>,
    content_sha256: String,
) -> Result<map_file::Model, DbErr> {
    let am = map_file::ActiveModel {
        map_id: Set(map_id),
        relative_path: Set(relative_path),
        content: Set(content),
        content_sha256: Set(content_sha256),
        ..Default::default()
    };

    map_file::Entity::insert(am)
        .on_conflict(
            OnConflict::columns([map_file::Column::MapId, map_file::Column::RelativePath])
                .update_columns([map_file::Column::Content, map_file::Column::ContentSha256])
                .to_owned(),
        )
        .exec_with_returning(db)
        .await
}

pub async fn delete(
    db: &DatabaseConnection,
    map_id: i32,
    relative_path: &str,
) -> Result<u64, DbErr> {
    let res = map_file::Entity::delete_many()
        .filter(map_file::Column::MapId.eq(map_id))
        .filter(map_file::Column::RelativePath.eq(relative_path))
        .exec(db)
        .await?;
    Ok(res.rows_affected)
}
