<script setup lang="ts">
const navItems = [
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
</script>

<template>
  <aside
    class="sticky top-0 z-10 max-h-full w-full shrink-0 overflow-y-auto border-b border-light-highlight-med bg-light-surface p-2 md:h-full md:w-58 md:border-r md:border-b-0 md:p-5 dark:border-dark-highlight-med dark:bg-dark-surface"
  >
    <h1 class="hidden text-3xl font-black md:block">Rincuma</h1>
    <hr class="hidden w-12 border-t-2 md:block" />
    <nav
      aria-label="Task views"
      class="flex gap-1 font-serif text-sm md:mt-8 md:block md:space-y-1"
    >
      <NuxtLink
        v-for="{ to, label, icon } in navItems"
        :to="to"
        class="flex items-center gap-2.5 rounded-md px-3 py-2.5 hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med"
        exact-active-class="bg-light-highlight-med/50 dark:bg-dark-highlight-med/50"
      >
        <Icon :name="icon" class="size-4" aria-hidden="true" />
        {{ label }}
      </NuxtLink>
    </nav>
  </aside>
</template>
