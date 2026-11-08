<script setup lang="ts">
defineProps<{ message: string }>();
</script>

<template>
  <div
    role="alert"
    class="flex items-start gap-2.5 rounded-lg border border-light-love/20 bg-light-love/5 px-3 py-3 font-sans text-sm leading-6 text-light-love dark:border-dark-love/20 dark:bg-dark-love/5 dark:text-dark-love"
  >
    <Icon name="carbon:warning-alt" class="mt-1 size-4 shrink-0" aria-hidden="true" />
    <p>{{ message }}</p>
  </div>
</template>
