export default defineNuxtRouteMiddleware(async () => {
  const { isLoggedIn } = useUserStore();
  const { loadDictionaries } = useDictionaries();
  const preferences = useNavigationPreferencesStore();

  if (isLoggedIn) {
    await Promise.all([loadDictionaries(), preferences.load()]);
  }
});
