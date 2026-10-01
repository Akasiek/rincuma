export default defineNuxtRouteMiddleware(async () => {
  const { fetchCurrentUser } = useAuth();

  await callOnce("session:init", () => fetchCurrentUser());
});
