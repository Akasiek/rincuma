export function useTask() {
  const requestFetch = useRequestFetch();

  async function createTask(requestBody: SaveTaskRequest): Promise<Task> {
    return await requestFetch<Task>("/api/tasks", {
      method: "POST",
      body: requestBody,
    });
  }

  return { createTask };
}
