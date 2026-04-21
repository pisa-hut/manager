use crate::entity::map_file;
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

pub async fn put(
    db: &DatabaseConnection,
    map_id: i32,
    relative_path: String,
    content: Vec<u8>,
    content_sha256: String,
) -> Result<map_file::Model, DbErr> {
    if let Some(existing) = get(db, map_id, &relative_path).await? {
        let mut am: map_file::ActiveModel = existing.into();
        am.content = Set(content);
        am.content_sha256 = Set(content_sha256);
        am.update(db).await
    } else {
        let am = map_file::ActiveModel {
            map_id: Set(map_id),
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
