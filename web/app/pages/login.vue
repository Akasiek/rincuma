<script setup lang="ts">
useHead({ title: "Log in" });

const { isLoggedIn } = useUserStore();

onMounted(async () => {
  if (isLoggedIn) {
    await navigateTo("/");
  }
});
</script>

<template>
  <main class="min-h-dvh bg-light-base text-light-text dark:bg-dark-base dark:text-dark-text">
    <div
      class="mx-auto flex min-h-dvh w-full max-w-lg flex-col justify-center px-5 py-10 sm:px-10 lg:max-w-6xl lg:px-12 lg:py-16"
    >
      <div class="grid items-center gap-10 lg:grid-cols-[1.1fr_1fr] lg:gap-20">
        <LoginHero />
        <LoginForm />
      </div>
    </div>
  </main>
</template>
