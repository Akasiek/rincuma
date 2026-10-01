export const useUserStore = defineStore("user", () => {
  const user = ref<User | null>(null);

  const isLoggedIn = computed(() => user.value !== null);

  function setUser(value: User) {
    user.value = value;
  }

  function clearUser() {
    user.value = null;
  }

  return { user, isLoggedIn, setUser, clearUser };
});
