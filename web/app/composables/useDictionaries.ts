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

  async function loadDictionaries() {
    if (store.loaded) {
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

  return { loadDictionaries };
}
