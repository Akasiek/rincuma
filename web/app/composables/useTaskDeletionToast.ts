import { useToastManager } from "reka-ui";
import type { Task } from "#shared/types/task";
import type { TaskDeletion } from "#shared/types/taskDeletion";

export function useTaskDeletionToast() {
  const toast = useToastManager();
  const { restoreTaskDeletion, refreshTaskLists } = useTask();
  const userStore = useUserStore();
  const pending = new Set<string>();

  function showDeletedTask(task: Task, deletion?: TaskDeletion) {
    toast.close(`task-completed:${task.owner_id}:${task.id}`);

    return toast.add({
      id: toastId(task),
      title: deletion ? "Task deleted" : "Task already deleted",
      description: task.name,
      status: "success",
      actionProps: deletion ? undoAction(task, deletion) : undefined,
    });
  }

  function showRestoreError(task: Task, deletion: TaskDeletion, status?: number) {
    const retry = status !== 404 && status !== 409;
    let description = "The undo period expired or this deletion was already restored.";
    if (status === 409) {
      description = "The undo period expired, or its parent or recurring series changed.";
    } else if (retry) {
      description = "Please try again.";
    }

    toast.update(toastId(task), {
      status: "error",
      title: "Could not undo deletion",
      description,
      actionProps: retry ? undoAction(task, deletion) : undefined,
    });
  }

  function undoAction(task: Task, deletion: TaskDeletion) {
    return {
      label: "Undo",
      altText: "Restore the deleted task and its subtasks",
      closeOnClick: false,
      onClick: () => {
        void undo(task, deletion);
      },
    };
  }

  async function undo(task: Task, deletion: TaskDeletion) {
    const operation = `${deletion.task_id}:${deletion.deleted_at}`;
    if (pending.has(operation) || userStore.user?.id !== task.owner_id) {
      return;
    }

    pending.add(operation);
    toast.update(toastId(task), {
      status: "loading",
      title: "Restoring task…",
      actionProps: undefined,
    });

    try {
      if (await restore(task, deletion)) {
        await refreshRestoredTask(task);
      }
    } finally {
      pending.delete(operation);
    }
  }

  async function restore(task: Task, deletion: TaskDeletion): Promise<boolean> {
    try {
      await restoreTaskDeletion(deletion);
      return true;
    } catch (cause) {
      const status = (cause as { response?: { status?: number } })?.response?.status;
      showRestoreError(task, deletion, status);
      return false;
    }
  }

  async function refreshRestoredTask(task: Task) {
    if (userStore.user?.id !== task.owner_id) {
      return;
    }

    toast.update(toastId(task), {
      status: "success",
      title: "Task restored",
      description: task.name,
      duration: 5000,
      actionProps: undefined,
    });

    try {
      await refreshTaskLists();
    } catch {
      toast.update(toastId(task), {
        description: "The task was restored. Refresh the list to see it.",
        duration: undefined,
      });
    }
  }

  function toastId(task: Task): string {
    return `task-deleted:${task.owner_id}:${task.id}`;
  }

  return { showDeletedTask };
}
