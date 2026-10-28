import type { CreateProjectRequest } from "#shared/types/project";
import type { CreateTagRequest } from "#shared/types/tag";

export function useDictionaries() {
  const store = useDictionariesStore();
  const userStore = useUserStore();
  const requestFetch = useRequestFetch();

  async function fetchAll<T>(url: string, query: Record<string, string> = {}): Promise<T[]> {
    const items: T[] = [];
    let offset = 0;

    while (true) {
      const page = await requestFetch<PageResponse<T>>(url, {
        query: {
          ...query,
          sort_by: "name",
          order: "asc",
          limit: 100,
          offset,
        },
      });

      items.push(...page.items);
      offset += page.items.length;

      if (offset >= page.total) {
        return items;
      }

      if (page.items.length === 0) {
        throw new Error("Cannot fetch full dictionary.");
      }
    }
  }

  async function loadDictionaries(force = false) {
    if (store.loaded && !force) {
      return;
    }

    const userId = userStore.user?.id;
    if (userId === undefined) {
      return;
    }

    const [tags, projects] = await Promise.all([
      fetchAll<Tag>("/api/tags"),
      fetchAll<Project>("/api/projects", { archived: "all" }),
    ]);

    if (userStore.user?.id !== userId) {
      return;
    }

    store.setDictionaries(tags, projects);
  }

  async function createProject(body: CreateProjectRequest): Promise<Project> {
    const project = await requestFetch<Project>("/api/projects", { method: "POST", body });
    if (userStore.user?.id === project.owner_id) {
      store.addProject(project);
    }

    return project;
  }

  async function createTag(body: CreateTagRequest): Promise<Tag> {
    const tag = await requestFetch<Tag>("/api/tags", { method: "POST", body });
    if (userStore.user?.id === tag.owner_id) {
      store.addTag(tag);
    }

    return tag;
  }

  async function updateProject(id: number, body: CreateProjectRequest): Promise<void> {
    const project = await requestFetch<Project>(`/api/projects/${id}`, { method: "PUT", body });
    if (userStore.user?.id === project.owner_id) {
      store.addProject(project);
    }
  }

  async function updateTag(id: number, body: CreateTagRequest): Promise<void> {
    const tag = await requestFetch<Tag>(`/api/tags/${id}`, { method: "PUT", body });
    if (userStore.user?.id === tag.owner_id) {
      store.addTag(tag);
    }
  }

  async function deleteRecord(kind: "project" | "tag", id: number): Promise<void> {
    const ownerId = userStore.user?.id;
    await requestFetch(`/api/${kind}s/${id}`, { method: "DELETE" });
    if (ownerId !== userStore.user?.id) {
      return;
    }

    if (kind === "project") {
      store.projects = store.projects.filter((project) => project.id !== id);
    } else {
      store.tags = store.tags.filter((tag) => tag.id !== id);
    }
  }

  return { loadDictionaries, createProject, createTag, updateProject, updateTag, deleteRecord };
}
