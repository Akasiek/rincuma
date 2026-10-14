CREATE INDEX "index_tasks_by_due_at" ON "tasks" ("due_at");
CREATE TABLE "notification_settings" (
    "owner_id" BIGINT NOT NULL,
    "provider" TEXT NOT NULL,
    "enabled" BOOLEAN NOT NULL,
    "url" TEXT NOT NULL,
    "authorization" TEXT,
    "created_at" TIMESTAMPTZ(6) NOT NULL,
    "updated_at" TIMESTAMPTZ(6) NOT NULL,
    PRIMARY KEY ("owner_id")
);
