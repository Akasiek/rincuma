import tailwindcss from "@tailwindcss/vite";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
  css: ["~/assets/css/main.css"],
  icon: {
    localApiEndpoint: "/_nuxt_icon",
  },
  vite: {
    plugins: [tailwindcss()],
  },

  $development: {
    routeRules: {
      "/api/**": {
        proxy: "http://127.0.0.1:7878/**",
      },
    },
  },

  modules: [
    "@nuxt/fonts",
    "@nuxt/hints",
    "@nuxt/icon",
    "@nuxt/image",
    "@nuxt/test-utils",
    "@pinia/nuxt",
    "@vee-validate/nuxt",
    "@vueuse/nuxt",
    "nuxt-yup",
  ],
});
