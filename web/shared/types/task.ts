export type TaskPriority = "none" | "low" | "medium" | "high" | "urgent";

export type RecurrenceFrequency = "daily" | "weekly" | "monthly";

export type RecurrenceEnd =
  | { mode: "never" }
  | { mode: "through"; date: string }
  | { mode: "after"; count: number };

export type RecurrencePattern =
  | { kind: "weekdays"; days: number[] }
  | { kind: "monthly_weekday"; ordinal: number; weekday: number };

export interface RecurrenceConfig {
  frequency: RecurrenceFrequency;
  interval: number;
  time_zone: string;
  end: RecurrenceEnd;
  pattern?: RecurrencePattern | null;
}

export interface TaskRecurrence extends RecurrenceConfig {
  anchor: string;
  occurrence_count: number;
}

export interface Task {
  id: number;
  name: string;
  description: string | null;
  due_at: string | null;
  completed_at: string | null;
  series_id: number | null;
  scheduled_at: string | null;
  recurrence: TaskRecurrence | null;
  priority: TaskPriority;
  owner_id: number;
  project_id: number | null;
  parent_id: number | null;
  tag_ids: number[];
  created_at: string;
  updated_at: string;
}

export interface SaveTaskRequest {
  name: string;
  description?: string | null;
  due_at?: string | null;
  priority?: TaskPriority | null;
  project_id?: number | null;
  parent_id?: number | null;
  tag_ids?: number[];
  recurrence?: RecurrenceConfig | null;
}
