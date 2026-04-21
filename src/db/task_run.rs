use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement,
};

/// Append a chunk to `task_run.log` without rewriting the entire column.
pub async fn append_log(
    db: &DatabaseConnection,
    run_id: i32,
    chunk: &str,
) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"UPDATE task_run SET log = COALESCE(log, '') || $1 WHERE id = $2"#,
        [chunk.into(), run_id.into()],
    ))
    .await?;
    Ok(())
}
