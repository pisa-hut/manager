use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "map_file")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub map_id: i32,
    pub relative_path: String,
    pub content: Vec<u8>,
    pub content_sha256: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::map::Entity",
        from = "Column::MapId",
        to = "super::map::Column::Id",
        on_delete = "Cascade"
    )]
    Map,
}

impl Related<super::map::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Map.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
