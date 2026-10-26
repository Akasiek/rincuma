import type { Ref } from "vue";
import type { Task } from "#shared/types/task";
import { useToastManager } from "reka-ui";

export function useTaskDeletion(task: Ref<Task>, onDeleted: () => void) {
  const { deleteTask, refreshTaskLists } = useTask();
  const toast = useToastManager();
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
      if (await deleteSelectedTask(selected)) {
        onDeleted();
        const notificationId = showDeletionToast(selected);
        await refreshAfterDeletion(notificationId);
      }
    } finally {
      deleting.value = false;
    }
  }

  async function deleteSelectedTask(selected: Task): Promise<boolean> {
    try {
      await deleteTask(selected.id);
      return true;
    } catch (cause) {
      const status = (cause as { response?: { status?: number } })?.response?.status;
      if (status === 404) {
        return true;
      }
      error.value =
        status === 409
          ? "The task or its subtasks changed. Check them and try again."
          : "Could not delete this task. Please try again.";
      return false;
    }
  }

  function showDeletionToast(selected: Task) {
    toast.close(`task-completed:${selected.owner_id}:${selected.id}`);
    return toast.add({
      title: "Task deleted",
      description: selected.name,
      status: "success",
      duration: 5000,
    });
  }

  async function refreshAfterDeletion(notificationId: ReturnType<typeof toast.add>) {
    try {
      await refreshTaskLists();
    } catch {
      toast.update(notificationId, {
        description: "The task was deleted. Refresh the list to see the change.",
        duration: 8000,
      });
    }
  }

  return { deleting, error, remove };
}
