-- Backfill existing users before enforcing NOT NULL; Toasty applies the default in the application.
ALTER TABLE "users" ADD COLUMN "is_admin" BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE "users" ALTER COLUMN "is_admin" DROP DEFAULT;
-- Existing installations: the oldest account becomes the admin.
UPDATE "users" SET "is_admin" = TRUE WHERE "id" = (SELECT MIN("id") FROM "users");
