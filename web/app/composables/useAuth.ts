type LoginCredentials = {
  email: string;
  password: string;
};

export function useAuth() {
  const userStore = useUserStore();
  const requestFetch = useRequestFetch();

  async function login(credentials: LoginCredentials) {
    const user = await $fetch<User>("/api/auth/login", {
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

  return { login, logout, fetchCurrentUser };
}
