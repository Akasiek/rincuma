<script setup lang="ts">
defineProps<{ id: string; description: string; open: boolean }>();
</script>

<template>
  <p
    v-show="open"
    :id="id"
    class="mt-3 border-l-2 border-light-highlight-med py-1 pl-3 text-sm leading-relaxed wrap-anywhere whitespace-pre-wrap text-light-subtle dark:border-dark-highlight-med dark:text-dark-subtle"
  >
    {{ description }}
  </p>
</template>
