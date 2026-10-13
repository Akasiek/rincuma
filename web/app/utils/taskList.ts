import type { Task } from "#shared/types/task";
import { formatDate, getDayKey } from "./dateTime";

const sectionOrder = new Map<string, number>([
  ["overdue", 0],
  ["unscheduled", 2],
  ["completed", 3],
]);

export interface TaskGroup {
  id: string;
  label: string;
  tasks: Task[];
}

export function groupTasks(tasks: Task[], now: Date, timeZone: string): TaskGroup[] {
  const groups = new Map<string, TaskGroup>();
  const today = getDayKey(now, timeZone);

  for (const task of tasks) {
    const section = getTaskSection(task, today, now, timeZone);
    const group = groups.get(section.id);

    if (group) {
      group.tasks.push(task);
    } else {
      groups.set(section.id, { ...section, tasks: [task] });
    }
  }

  return [...groups.values()].sort(compareTaskGroups);
}

function getTaskSection(task: Task, today: string, now: Date, timeZone: string) {
  if (task.completed_at) {
    return { id: "completed", label: "Completed" };
  }

  if (!task.due_at) {
    return { id: "unscheduled", label: "No date" };
  }

  const date = new Date(task.due_at);
  const day = getDayKey(date, timeZone);
  if (day < today) {
    return { id: "overdue", label: "Overdue" };
  }
  if (day === today) {
    return { id: today, label: "Today" };
  }

  const label = formatDate(date, now, timeZone, {
    weekday: "long",
    day: "numeric",
    month: "long",
  });

  return { id: day, label };
}

function compareTaskGroups(left: TaskGroup, right: TaskGroup): number {
  const orderDifference = getSectionOrder(left.id) - getSectionOrder(right.id);

  return orderDifference !== 0 ? orderDifference : left.id.localeCompare(right.id);
}

function getSectionOrder(id: string): number {
  return sectionOrder.get(id) ?? 1;
}
