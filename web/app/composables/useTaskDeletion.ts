import type { Ref } from "vue";
import type { Task } from "#shared/types/task";
import type { TaskDeletion } from "#shared/types/taskDeletion";
import { useToastManager } from "reka-ui";

export function useTaskDeletion(task: Ref<Task>, onDeleted: () => void) {
  const { deleteTask, refreshTaskLists } = useTask();
  const toast = useToastManager();
  const { showDeletedTask } = useTaskDeletionToast();
  const deleting = ref(false);
  const error = ref("");

  async function remove() {
    if (deleting.value) {
      return;
    }
    const selected = task.value;
    deleting.value = true;
    error.value = "";
    try {
      const result = await deleteSelectedTask(selected);
      if (result) {
        onDeleted();
        const notificationId = showDeletedTask(selected, result.undo);
        await refreshAfterDeletion(notificationId);
      }
    } finally {
      deleting.value = false;
    }
  }

  async function deleteSelectedTask(selected: Task): Promise<{ undo?: TaskDeletion } | null> {
    try {
      return { undo: await deleteTask(selected.id) };
    } catch (cause) {
      const status = (cause as { response?: { status?: number } })?.response?.status;
      if (status === 404) {
        return {};
      }
      error.value =
        status === 409
          ? "The task or its subtasks changed. Check them and try again."
          : "Could not delete this task. Please try again.";

      return null;
    }
  }

  async function refreshAfterDeletion(notificationId: ReturnType<typeof toast.add>) {
    try {
      await refreshTaskLists();
    } catch {
      const notification = toast.toasts.value.find((item) => item.id === notificationId);
      if (
        notification?.title !== "Task deleted" &&
        notification?.title !== "Task already deleted"
      ) {
        return;
      }

      toast.update(notificationId, {
        description: "The task was deleted. Refresh the list to see the change.",
      });
    }
  }

  return { deleting, error, remove };
}
