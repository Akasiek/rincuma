import type { TaskFilters, TaskListQuery } from "#shared/types/taskFilters";
import { getCalendarDate } from "./dateTime";

export function resolveTaskFilters(
  filters: TaskFilters,
  now: Date,
  timeZone: string,
): TaskListQuery {
  const { period, ...query } = filters;
  if (!period) {
    return query;
  }

  const start = getCalendarDate(now, timeZone);
  const days = period === "today" ? 1 : 7;
  const end = start.add({ days });

  return {
    ...query,
    due_from: start.toDate(timeZone).toISOString(),
    due_before: end.toDate(timeZone).toISOString(),
  };
}
