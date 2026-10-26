CREATE TYPE "series_pattern" AS ENUM ('weekdays', 'monthly_weekday');
ALTER TABLE "task_series" ADD COLUMN "pattern_mask" BIGINT;
ALTER TABLE "task_series" ADD COLUMN "pattern" "series_pattern";
ALTER TABLE "task_series" ADD COLUMN "pattern_weekday" BIGINT;
ALTER TABLE "task_series" ADD COLUMN "pattern_ordinal" INTEGER;
