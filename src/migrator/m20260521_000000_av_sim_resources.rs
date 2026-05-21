use sea_orm_migration::prelude::*;

/// Add per-image resource hints (cpu_count, memory_gb, gpu_count) to
/// the `av` and `simulator` tables. The scheduler sums an (AV, Sim)
/// task's hints to size each SLURM allocation, and PostgREST exposes
/// them so the Resources page can edit them.
///
/// All-zero defaults keep existing rows valid; the scheduler clamps
/// to 1 CPU minimum at submit time so a forgotten value can't queue
/// a zero-CPU job.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260521_000000_av_sim_resources"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            ALTER TABLE av
                ADD COLUMN cpu_count int NOT NULL DEFAULT 0,
                ADD COLUMN memory_gb int NOT NULL DEFAULT 0,
                ADD COLUMN gpu_count int NOT NULL DEFAULT 0;

            ALTER TABLE simulator
                ADD COLUMN cpu_count int NOT NULL DEFAULT 0,
                ADD COLUMN memory_gb int NOT NULL DEFAULT 0,
                ADD COLUMN gpu_count int NOT NULL DEFAULT 0;

            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            ALTER TABLE av
                DROP COLUMN IF EXISTS cpu_count,
                DROP COLUMN IF EXISTS memory_gb,
                DROP COLUMN IF EXISTS gpu_count;

            ALTER TABLE simulator
                DROP COLUMN IF EXISTS cpu_count,
                DROP COLUMN IF EXISTS memory_gb,
                DROP COLUMN IF EXISTS gpu_count;

            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }
}
