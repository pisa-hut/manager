use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "concrete_run")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub task_id: i32,
    pub task_run_id: i32,
    pub concrete_key: String,
    pub status: String,
    pub test_outcome: String,
    pub reason: Option<String>,
    pub stop_condition: Option<String>,
    pub params: Option<Json>,
    pub final_sim_time_ms: Option<f64>,
    pub wall_time_ms: Option<f64>,
    pub total_steps: Option<i32>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Task,
    #[sea_orm(
        belongs_to = "super::task_run::Entity",
        from = "Column::TaskRunId",
        to = "super::task_run::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    TaskRun,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<super::task_run::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskRun.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
