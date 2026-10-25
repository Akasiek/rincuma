<script setup lang="ts">
const applicationItems = [
  {
    to: "/",
    icon: "carbon:task",
    label: "All tasks",
  },
  {
    to: "/this-week",
    icon: "carbon:calendar",
    label: "Next 7 days",
  },
];

const personalItems = [
  {
    to: "/settings",
    icon: "carbon:settings",
    label: "Settings",
  },
];
</script>

<template>
  <aside
    class="sticky top-0 z-10 flex max-h-full w-full shrink-0 flex-col overflow-y-auto border-b border-light-highlight-med bg-light-surface p-2 md:h-full md:w-58 md:border-r md:border-b-0 md:px-4 md:py-5 lg:w-64 xl:w-72 dark:border-dark-highlight-med dark:bg-dark-surface"
  >
    <div class="hidden md:block">
      <h1 class="text-3xl font-black">Rincuma</h1>
      <hr class="w-12 border-t-2" />
    </div>
    <NavigationSection label="Task navigation" :items="applicationItems" class="md:mt-8" />
    <div
      class="mt-2 border-t border-light-highlight-med pt-2 md:mt-auto md:pt-4 dark:border-dark-highlight-med"
    >
      <NavigationSection label="Personal navigation" :items="personalItems" />
      <NavigationProfileMenu class="mt-1" />
    </div>
  </aside>
</template>
