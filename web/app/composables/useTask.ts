export function useTask() {
  const requestFetch = useRequestFetch();
  const userStore = useUserStore();
  const taskListKey = computed(() => `tasks:${userStore.user?.id ?? "guest"}`);

  async function listAllTasks(): Promise<Task[]> {
    const tasks: Task[] = [];
    let offset = 0;

    while (true) {
      const page = await requestFetch<PageResponse<Task>>("/api/tasks", {
        query: { status: "all", limit: 100, offset },
      });

      tasks.push(...page.items);
      offset += page.items.length;

      if (offset >= page.total) {
        return tasks;
      }

      if (page.items.length === 0) {
        throw new Error("Cannot fetch the full task list.");
      }
    }
  }

  async function createTask(requestBody: SaveTaskRequest): Promise<Task> {
    const task = await requestFetch<Task>("/api/tasks", {
      method: "POST",
      body: requestBody,
    });

    await refreshNuxtData(taskListKey.value);
    return task;
  }

  return { createTask, listAllTasks, taskListKey };
}
