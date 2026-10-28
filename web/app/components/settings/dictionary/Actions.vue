<script setup lang="ts">
defineProps<{ name: string }>();
const emit = defineEmits<{ edit: []; delete: [] }>();
</script>

<template>
  <div class="flex shrink-0 items-center gap-1">
    <button
      type="button"
      :aria-label="`Edit ${name}`"
      :title="`Edit ${name}`"
      class="flex min-h-9 min-w-9 cursor-pointer items-center justify-center gap-1.5 rounded-md px-2 font-sans text-xs text-light-subtle transition-colors hover:bg-light-highlight-med hover:text-light-text dark:text-dark-subtle dark:hover:bg-dark-highlight-med dark:hover:text-dark-text"
      @click="emit('edit')"
    >
      <Icon name="carbon:edit" class="size-4" aria-hidden="true" />
      <span class="hidden sm:inline">Edit</span>
    </button>
    <button
      type="button"
      :aria-label="`Delete ${name}`"
      :title="`Delete ${name}`"
      class="flex min-h-9 min-w-9 cursor-pointer items-center justify-center gap-1.5 rounded-md px-2 font-sans text-xs text-light-love transition-colors hover:bg-light-love/10 dark:text-dark-love dark:hover:bg-dark-love/10"
      @click="emit('delete')"
    >
      <Icon name="carbon:trash-can" class="size-4" aria-hidden="true" />
      <span class="hidden sm:inline">Delete</span>
    </button>
  </div>
</template>
