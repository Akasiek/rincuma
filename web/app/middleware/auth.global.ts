const publicPaths = new Set(["/login"]);

export default defineNuxtRouteMiddleware((to) => {
  const path = to.path.replace(/\/+$/, "") || "/";

  if (publicPaths.has(path)) {
    return;
  }

  const userStore = useUserStore();

  if (!userStore.isLoggedIn) {
    return navigateTo("/login", { replace: true });
  }
});
