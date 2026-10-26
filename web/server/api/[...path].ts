export default defineEventHandler((event) => {
  const config = useRuntimeConfig(event);
  const url = getRequestURL(event);
  const base = config.apiBase.replace(/\/+$/, "");
  const path = url.pathname.slice("/api".length);

  return proxyRequest(event, `${base}${path}${url.search}`);
});
