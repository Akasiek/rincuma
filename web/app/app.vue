<script setup lang="ts">
const userStore = useUserStore();
const { open: confirmationOpen, saving, finish } = useNavigationConfirmation();

useHead({
  titleTemplate: (title) => (title ? `${title} · Rincuma` : "Rincuma"),
});
</script>

<template>
  <div class="bg-light-base font-serif text-light-text dark:bg-dark-base dark:text-dark-text">
    <NuxtPage />
    <TaskCreatorLauncher v-if="userStore.isLoggedIn" />
    <UnsavedChangesModal v-model:open="confirmationOpen" :saving="saving" @decision="finish" />
  </div>
</template>
