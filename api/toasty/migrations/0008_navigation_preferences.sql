CREATE TABLE "navigation_preferences" (
    "owner_id" BIGINT NOT NULL,
    "projects_open" BOOLEAN,
    "tags_open" BOOLEAN,
    PRIMARY KEY ("owner_id")
);
