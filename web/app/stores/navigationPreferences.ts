import type { NavigationPreferences, NavigationSection } from "#shared/types/navigationPreferences";

export const useNavigationPreferencesStore = defineStore("navigation-preferences", () => {
  const userStore = useUserStore();
  const requestFetch = useRequestFetch();
  const preferences = ref<NavigationPreferences>({ projects_open: null, tags_open: null });
  const loadedFor = ref<number | null>(null);
  const saving = reactive({ projects: false, tags: false });

  async function load() {
    const ownerId = userStore.user?.id;
    if (ownerId === undefined || loadedFor.value === ownerId) {
      return;
    }
    const values = await requestFetch<NavigationPreferences>("/api/preferences/navigation");
    if (userStore.user?.id === ownerId) {
      preferences.value = values;
      loadedFor.value = ownerId;
    }
  }

  function isOpen(section: NavigationSection, count: number) {
    return preferences.value[`${section}_open`] ?? count <= 6;
  }

  async function setOpen(section: NavigationSection, open: boolean) {
    if (saving[section]) {
      return;
    }

    const ownerId = userStore.user?.id;
    const key = `${section}_open` as const;
    const previous = preferences.value[key];
    preferences.value[key] = open;
    saving[section] = true;

    try {
      await requestFetch(`/api/preferences/navigation/${section}`, {
        method: "PUT",
        body: { open },
      });
    } catch (error) {
      if (userStore.user?.id === ownerId) {
        preferences.value[key] = previous;
      }
      throw error;
    } finally {
      saving[section] = false;
    }
  }

  function clear() {
    preferences.value = { projects_open: null, tags_open: null };
    loadedFor.value = null;
  }

  return { preferences, saving, load, isOpen, setOpen, clear };
});
