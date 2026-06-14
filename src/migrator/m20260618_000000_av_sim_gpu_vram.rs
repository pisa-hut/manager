use sea_orm_migration::prelude::*;

/// Add a per-image VRAM hint (gpu_vram_mb) to `av` and `simulator`.
/// When a task needs no whole GPU (gpu_count == 0) but declares VRAM,
/// the scheduler requests `--gres=shard:ceil(vram_mb/1024)` so several
/// light tasks (e.g. PLANT ~1 GB) share one physical GPU via SLURM
/// sharding. Zero default keeps existing rows on the whole-GPU path.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260618_000000_av_sim_gpu_vram"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            r#"
            ALTER TABLE av
                ADD COLUMN gpu_vram_mb int NOT NULL DEFAULT 0;

            ALTER TABLE simulator
                ADD COLUMN gpu_vram_mb int NOT NULL DEFAULT 0;

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
                DROP COLUMN IF EXISTS gpu_vram_mb;

            ALTER TABLE simulator
                DROP COLUMN IF EXISTS gpu_vram_mb;

            NOTIFY pgrst, 'reload schema';
            "#,
        )
        .await?;
        Ok(())
    }
}
