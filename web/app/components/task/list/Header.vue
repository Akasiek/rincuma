<script setup lang="ts">
defineProps<{
  title: string;
  total: number;
  activeCount: number;
  completedCount: number;
  loaded: boolean;
}>();
</script>

<template>
  <header class="mb-8">
    <div class="flex items-center gap-3">
      <h1 class="text-2xl font-black tracking-tight sm:text-3xl">
        {{ title }}
      </h1>
      <span
        v-if="loaded"
        class="mb-0.5 rounded-md bg-light-highlight-med/5 px-2 py-0.5 font-mono text-xs text-light-subtle tabular-nums dark:bg-dark-highlight-med/50 dark:text-dark-subtle"
      >
        {{ total }}
      </span>
    </div>

    <p
      v-if="loaded"
      class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 font-sans text-xs text-light-subtle dark:text-dark-subtle"
    >
      <span class="inline-flex items-center gap-1.5">
        <Icon name="carbon:circle-dash" class="size-3.5" aria-hidden="true" />
        {{ activeCount }} active
      </span>
      <span class="inline-flex items-center gap-1.5">
        <Icon name="carbon:checkmark" class="size-3.5" aria-hidden="true" />
        {{ completedCount }} completed
      </span>
    </p>
  </header>
</template>
