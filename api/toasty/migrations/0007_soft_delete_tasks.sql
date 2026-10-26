ALTER TABLE "tasks" ADD COLUMN "deleted_at" TIMESTAMPTZ(6);
ALTER TABLE "tasks" ADD COLUMN "stopped_series_id" BIGINT;
ALTER TABLE "tasks" ADD COLUMN "deletion_root_id" BIGINT;
CREATE INDEX "index_tasks_by_deleted_at" ON "tasks" ("deleted_at");
CREATE INDEX "index_tasks_by_deletion_root_id" ON "tasks" ("deletion_root_id");
