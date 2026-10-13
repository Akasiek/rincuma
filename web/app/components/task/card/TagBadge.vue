<script setup lang="ts">
defineProps<{ tag: Tag }>();
</script>

<template>
  <div
    class="inline-flex min-w-0 items-center gap-1 rounded px-1.5 leading-5 text-light-subtle dark:text-dark-subtle"
    :style="{
      backgroundColor: `color-mix(in srgb, ${tag.color || 'currentColor'} 8%, transparent)`,
    }"
    :title="`Tag: ${tag.name}`"
  >
    <span
      v-if="tag.color"
      class="size-1.5 shrink-0 rounded-full"
      :style="{ backgroundColor: tag.color }"
      aria-hidden="true"
    />
    <span class="max-w-32 truncate font-sans">{{ tag.name }}</span>
  </div>
</template>
