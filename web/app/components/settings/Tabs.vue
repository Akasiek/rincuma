<script setup lang="ts">
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { SettingsProjectsPanel, SettingsTagsPanel, SettingsNotificationsPanel } from "#components";

const tabs = [
  {
    value: "projects",
    label: "Projects",
    icon: "carbon:folder",
    panel: SettingsProjectsPanel,
  },
  {
    value: "tags",
    label: "Tags",
    icon: "carbon:tag",
    panel: SettingsTagsPanel,
  },
  {
    value: "notifications",
    label: "Notifications",
    icon: "carbon:notification",
    panel: SettingsNotificationsPanel,
  },
] as const;
</script>

<template>
  <TabsRoot :default-value="tabs[0].value">
    <TabsList
      aria-label="Settings categories"
      class="mb-8 flex flex-wrap gap-x-6 gap-y-3 border-b border-light-highlight-med dark:border-dark-highlight-med"
    >
      <TabsTrigger
        v-for="tab in tabs"
        :key="tab.value"
        :value="tab.value"
        class="-mb-px flex cursor-pointer items-center gap-2 border-b-2 border-transparent px-1 pb-3 font-sans text-sm text-light-subtle transition-colors data-[state=active]:border-light-pine data-[state=active]:text-light-pine dark:text-dark-subtle dark:data-[state=active]:border-dark-foam dark:data-[state=active]:text-dark-foam"
      >
        <Icon :name="tab.icon" class="size-4" aria-hidden="true" />
        {{ tab.label }}
      </TabsTrigger>
    </TabsList>
    <TabsContent v-for="tab in tabs" :key="tab.value" :value="tab.value" class="rounded-md">
      <component :is="tab.panel" />
    </TabsContent>
  </TabsRoot>
</template>
