type Credentials = {
  email: string;
  password: string;
};

export function useAuth() {
  const userStore = useUserStore();
  const dictionariesStore = useDictionariesStore();
  const navigationPreferences = useNavigationPreferencesStore();
  const requestFetch = useRequestFetch();

  async function login(credentials: Credentials) {
    const user = await $fetch<User>("/api/auth/login", {
      method: "POST",
      body: credentials,
    });

    userStore.setUser(user);
  }

  async function register(credentials: Credentials) {
    const user = await $fetch<User>("/api/auth/register", {
      method: "POST",
      body: credentials,
    });

    userStore.setUser(user);
  }

  async function logout() {
    await $fetch("/api/auth/logout", {
      method: "POST",
    });

    userStore.clearUser();
    dictionariesStore.clear();
    navigationPreferences.clear();
    clearNuxtData();
  }

  async function fetchCurrentUser() {
    try {
      const user = await requestFetch<User>("/api/auth/me");

      userStore.setUser(user);
      return user;
    } catch (error) {
      const status = (error as { response?: { status: number } }).response?.status;

      if (status === 401) {
        userStore.clearUser();
        return null;
      }

      throw error;
    }
  }

  return { login, register, logout, fetchCurrentUser };
}
