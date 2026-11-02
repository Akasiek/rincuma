<script setup lang="ts">
const emit = defineEmits<{ navigate: [] }>();
const applicationItems = [
  { to: "/", icon: "carbon:task", label: "All tasks" },
  { to: "/this-week", icon: "carbon:calendar", label: "Next 7 days" },
];
const personalItems = [{ to: "/settings", icon: "carbon:settings", label: "Settings" }];
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex shrink-0 items-center justify-between gap-3 px-4 py-5">
      <div>
        <h1 class="text-3xl font-black">Rincuma</h1>
        <hr class="w-12 border-t-2" />
      </div>
      <slot name="actions" />
    </div>

    <AppScrollArea class="min-h-0 flex-1" label="Navigation">
      <div class="space-y-5 px-3 pb-4">
        <NavigationSection
          label="Task navigation"
          :items="applicationItems"
          @navigate="emit('navigate')"
        />
        <div class="space-y-2">
          <NavigationDictionarySection section="projects" @navigate="emit('navigate')" />
          <NavigationDictionarySection section="tags" @navigate="emit('navigate')" />
        </div>
      </div>
    </AppScrollArea>

    <div
      class="shrink-0 border-t border-light-highlight-med px-3 py-4 dark:border-dark-highlight-med"
    >
      <NavigationSection
        label="Personal navigation"
        :items="personalItems"
        @navigate="emit('navigate')"
      />
      <NavigationProfileMenu class="mt-1" />
    </div>
  </div>
</template>
