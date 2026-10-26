<script setup lang="ts">
import { ToastProvider } from "reka-ui";
const userStore = useUserStore();
const { isTimeZoneReady } = useTimeZone();
const isPageReady = computed(() => !userStore.isLoggedIn || isTimeZoneReady.value);
const { open: confirmationOpen, saving, finish } = useNavigationConfirmation();

useHead({
  titleTemplate: (title) => (title ? `${title} · Rincuma` : "Rincuma"),
});
</script>

<template>
  <ToastProvider :duration="8000">
    <div class="bg-light-base font-serif text-light-text dark:bg-dark-base dark:text-dark-text">
      <NuxtPage :key="isPageReady ? 'ready' : 'waiting-for-time-zone'" v-slot="{ Component }">
        <component :is="Component" v-if="isPageReady" />
      </NuxtPage>
      <template v-if="isPageReady">
        <TaskCreatorLauncher v-if="userStore.isLoggedIn" />
        <TaskEditorHost v-if="userStore.isLoggedIn" />
      </template>
      <AppToaster />
      <UnsavedChangesModal v-model:open="confirmationOpen" :saving="saving" @decision="finish" />
    </div>
  </ToastProvider>
</template>
