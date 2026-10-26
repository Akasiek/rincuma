import type { SaveTaskRequest } from "#shared/types/task";

export function taskFormValues(task?: SaveTaskRequest): SaveTaskRequest {
  return {
    name: task?.name ?? "",
    description: task?.description ?? null,
    due_at: task?.due_at ?? null,
    priority: task?.priority ?? "none",
    project_id: task?.project_id ?? null,
    parent_id: task?.parent_id ?? null,
    tag_ids: [...(task?.tag_ids ?? [])],
    recurrence: task?.recurrence
      ? {
          frequency: task.recurrence.frequency,
          interval: task.recurrence.interval,
          time_zone: task.recurrence.time_zone,
          end: { ...task.recurrence.end },
        }
      : task?.recurrence,
  };
}
