<script setup lang="ts">
import type { NotificationSettings } from "#shared/types/notification";

const userStore = useUserStore();
const { getSettings } = useNotificationSettings();
const settingsKey = computed(() => `notification-settings:${userStore.user?.id ?? "guest"}`);
const { data, status, error, refresh } = await useAsyncData(settingsKey, async () => ({
  settings: await getSettings(),
}));

function updateSettings(settings: NotificationSettings) {
  data.value = { settings };
}
</script>

<template>
  <section aria-labelledby="notification-settings-title" :aria-busy="status === 'pending'">
    <div class="mb-6">
      <h2 id="notification-settings-title" class="text-xl font-bold">Notifications</h2>
      <p
        class="mt-2 max-w-xl font-sans text-sm leading-relaxed text-light-subtle dark:text-dark-subtle"
      >
        Get a reminder when a task is due. Connect your own notification service to choose where it
        arrives.
      </p>
    </div>

    <div
      v-if="error"
      class="rounded-lg border border-light-love/30 bg-light-surface p-6 dark:border-dark-love/30 dark:bg-dark-surface"
    >
      <p role="alert" class="font-sans text-sm text-light-love dark:text-dark-love">
        Could not load your notification settings.
      </p>
      <Button size="sm" class="mt-4 font-sans" @click="refresh()">Try again</Button>
    </div>
    <SettingsNotificationsForm v-else-if="data" :settings="data.settings" @saved="updateSettings" />
  </section>
</template>
