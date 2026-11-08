import type { Instance } from "#shared/types/instance";

export const useInstanceStore = defineStore("instance", () => {
  const requestFetch = useRequestFetch();
  const instance = ref<Instance | null>(null);
  const registrationEnabled = computed(() => instance.value?.registration_enabled ?? false);

  async function load() {
    if (instance.value) {
      return;
    }

    try {
      instance.value = await requestFetch<Instance>("/api/instance");
    } catch {
      // Treat an unreachable endpoint as disabled registration and retry on the next load.
      instance.value = null;
    }
  }

  return { instance, registrationEnabled, load };
});
