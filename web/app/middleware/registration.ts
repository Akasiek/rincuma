export default defineNuxtRouteMiddleware(async () => {
  if (useUserStore().isLoggedIn) {
    return navigateTo("/", { replace: true });
  }

  const instance = useInstanceStore();
  await instance.load();

  if (!instance.registrationEnabled) {
    return navigateTo("/login", { replace: true });
  }
});
