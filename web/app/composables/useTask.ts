import type { TaskListQuery } from "#shared/types/taskFilters";

export function useTask() {
  const requestFetch = useRequestFetch();
  const userStore = useUserStore();

  const taskListKeyPrefix = computed(() => `tasks:${userStore.user?.id ?? "guest"}`);
  const registeredTaskListKeys = useState<string[]>("task-list-keys", () => []);

  function getTaskListKey(filters: TaskListQuery = {}): string {
    const query = { ...filters, status: filters.status ?? "all" };
    const entries = Object.entries(query)
      .filter(([, value]) => value !== undefined)
      .sort(([left], [right]) => left.localeCompare(right));

    return entries.length === 1 && query.status === "all"
      ? taskListKeyPrefix.value
      : `${taskListKeyPrefix.value}:${JSON.stringify(entries)}`;
  }

  async function refreshTaskLists(exceptKey?: string) {
    const prefix = taskListKeyPrefix.value;
    const keys = registeredTaskListKeys.value.filter(
      (key) => (key === prefix || key.startsWith(`${prefix}:`)) && key !== exceptKey,
    );

    clearNuxtData(keys);
    await refreshNuxtData(keys);
  }

  async function listTasks(filters: TaskListQuery = {}): Promise<Task[]> {
    const key = getTaskListKey(filters);
    if (!registeredTaskListKeys.value.includes(key)) {
      registeredTaskListKeys.value.push(key);
    }

    const tasks: Task[] = [];
    let offset = 0;

    while (true) {
      const page = await requestFetch<PageResponse<Task>>("/api/tasks", {
        query: { ...filters, status: filters.status ?? "all", limit: 100, offset },
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

  async function listAllTasks(): Promise<Task[]> {
    return listTasks();
  }

  async function createTask(requestBody: SaveTaskRequest): Promise<Task> {
    const task = await requestFetch<Task>("/api/tasks", {
      method: "POST",
      body: requestBody,
    });
    await refreshTaskLists();

    return task;
  }

  async function completeTask(id: number): Promise<Task> {
    return await requestFetch<Task>(`/api/tasks/${id}/complete`, { method: "POST" });
  }

  async function reopenTask(id: number): Promise<Task> {
    return await requestFetch<Task>(`/api/tasks/${id}/complete`, { method: "DELETE" });
  }

  return {
    createTask,
    completeTask,
    reopenTask,
    listTasks,
    listAllTasks,
    taskListKeyPrefix,
    getTaskListKey,
    refreshTaskLists,
  };
}
