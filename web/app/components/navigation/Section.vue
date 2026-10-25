<script setup lang="ts">
defineProps<{
  label: string;
  items: { to: string; icon: string; label: string }[];
}>();
</script>

<template>
  <nav :aria-label="label" class="flex gap-1 font-serif text-sm md:block md:space-y-1">
    <NuxtLink
      v-for="{ to, label: itemLabel, icon } in items"
      :key="to"
      :to="to"
      class="flex items-center gap-2.5 rounded-md px-3 py-2.5 hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med"
      exact-active-class="bg-light-highlight-med/50 dark:bg-dark-highlight-med/50"
    >
      <Icon :name="icon" class="size-4 shrink-0" aria-hidden="true" />
      {{ itemLabel }}
    </NuxtLink>
  </nav>
</template>
