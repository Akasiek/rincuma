<script setup lang="ts">
const userStore = useUserStore();
</script>

<template>
  <div class="bg-light-base font-serif text-light-text dark:bg-dark-base dark:text-dark-text">
    <NuxtPage />
    <TaskCreatorLauncher v-if="userStore.isLoggedIn" />
  </div>
</template>
