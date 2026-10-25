<script setup lang="ts">
defineProps<{ failed: boolean; empty: boolean }>();
const emit = defineEmits<{ retry: [] }>();
</script>

<template>
  <div v-if="failed" role="alert" class="space-y-3 py-8">
    <p class="font-sans text-sm text-light-love dark:text-dark-love">
      Could not load tasks. Please try again.
    </p>
    <Button @click="emit('retry')">Try again</Button>
  </div>

  <div v-else-if="empty" class="py-12 text-center">
    <Icon
      name="carbon:task"
      class="size-8 text-light-subtle dark:text-dark-subtle"
      aria-hidden="true"
    />
    <h2 class="mt-3 text-base font-medium">No tasks in this view</h2>
    <p class="mt-1.5 font-sans text-xs text-light-subtle dark:text-dark-subtle">
      Add a task above, or press C. Tasks appear here when they match this view.
    </p>
  </div>

  <slot v-else />
</template>
