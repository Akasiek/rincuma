import type { TaskPriority } from "./task";

export type TaskStatus = "all" | "active" | "completed" | "overdue" | "upcoming";
export type TaskSort = "name" | "due_at" | "created_at" | "updated_at";
export type TaskPeriod = "today" | "next-seven-days";

export interface TaskListQuery {
  status?: TaskStatus;
  project_id?: number;
  tag_id?: number;
  priority?: TaskPriority;
  due_from?: string;
  due_before?: string;
  sort_by?: TaskSort;
  order?: "asc" | "desc";
}

export interface TaskFilters extends TaskListQuery {
  period?: TaskPeriod;
}
