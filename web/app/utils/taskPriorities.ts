import type { TaskPriority } from "#shared/types/task";

export const taskPriorities = {
  none: { label: "None", color: "text-light-subtle dark:text-dark-subtle" },
  low: { label: "Low", color: "text-light-pine dark:text-dark-foam" },
  medium: { label: "Medium", color: "text-light-gold dark:text-dark-gold" },
  high: { label: "High", color: "text-light-rose dark:text-dark-rose" },
  urgent: { label: "Urgent", color: "text-light-love dark:text-dark-love" },
} satisfies Record<TaskPriority, { label: string; color: string }>;

export const taskPriorityOptions = Object.entries(taskPriorities).map(([value, priority]) => ({
  value: value as TaskPriority,
  ...priority,
}));
