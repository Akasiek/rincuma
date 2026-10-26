import { useToastManager } from "reka-ui";
import type { Task } from "#shared/types/task";

export function useTaskCompletionToast() {
  const toast = useToastManager();
  const { reopenTask, refreshTaskLists } = useTask();
  const userStore = useUserStore();
  const pending = new Set<number>();

  function showCompletedTask(task: Task) {
    toast.add({
      id: toastId(task),
      title: "Task completed",
      description: task.name,
      status: "success",
      actionProps: {
        label: "Undo",
        altText: "Undo completion, or reopen the task from the list later",
        closeOnClick: false,
        onClick: () => {
          void undo(task);
        },
      },
    });
  }

  async function undo(task: Task) {
    const id = toastId(task);
    if (pending.has(task.id) || userStore.user?.id !== task.owner_id) {
      return;
    }

    pending.add(task.id);
    toast.update(id, { status: "loading", title: "Reopening task…", actionProps: undefined });

    try {
      await reopenTask(task.id);
    } catch (error) {
      const status = (error as { response?: { status?: number } })?.response?.status;
      toast.update(id, {
        status: "error",
        title: "Could not undo completion",
        description:
          status === 409
            ? "Its series or next occurrence has changed."
            : "Please try reopening the task from the list.",
      });
      pending.delete(task.id);

      return;
    }

    toast.update(id, {
      status: "success",
      title: "Task reopened",
      description: task.name,
      duration: 5000,
    });

    try {
      await refreshTaskLists();
    } catch {
      toast.update(id, {
        description: "The task was reopened. Refresh the list to see it.",
        duration: undefined,
      });
    } finally {
      pending.delete(task.id);
    }
  }

  function dismissCompletedTask(task: Task) {
    toast.close(toastId(task));
  }

  function toastId(task: Task): string {
    return `task-completed:${task.owner_id}:${task.id}`;
  }

  return { showCompletedTask, dismissCompletedTask };
}
