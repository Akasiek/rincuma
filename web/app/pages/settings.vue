<script setup lang="ts">
defineOptions({ name: "SettingsPage" });
useHead({ title: "Settings" });
</script>

<template>
  <AppContentPage title="Settings">
    <SettingsTabs />
  </AppContentPage>
</template>
