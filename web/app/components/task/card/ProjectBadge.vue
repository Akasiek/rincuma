<script setup lang="ts">
defineProps<{ project: Project }>();
</script>

<template>
  <span
    class="inline-flex min-w-0 items-center gap-1 rounded px-1.5 text-light-subtle dark:text-dark-subtle"
    :style="{
      backgroundColor: `color-mix(in srgb, ${project.color || 'currentColor'} 8%, transparent)`,
    }"
    :title="`Project: ${project.name}`"
  >
    <Icon
      name="carbon:folder"
      class="size-3 shrink-0"
      :style="{ color: project.color || undefined }"
      aria-hidden="true"
    />
    <span class="max-w-36 truncate font-sans">{{ project.name }}</span>
  </span>
</template>
