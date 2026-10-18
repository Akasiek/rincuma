<script setup lang="ts">
const props = defineProps<{
  id: string;
  label: string;
  description?: string;
  error?: string;
  optional?: boolean;
}>();

const describedby = computed(() => {
  const ids: string[] = [];
  if (props.description) {
    ids.push(`${props.id}-help`);
  }
  if (props.error) {
    ids.push(`${props.id}-error`);
  }
  return ids.join(" ") || undefined;
});
</script>

<template>
  <div>
    <div class="mb-2 flex flex-wrap items-center justify-between gap-2">
      <FormLabel :for="id" :optional="optional">{{ label }}</FormLabel>
      <slot name="actions" />
    </div>
    <slot :describedby="describedby" :invalid="!!error" />
    <p
      v-if="description"
      :id="`${id}-help`"
      class="mt-2 font-sans text-xs leading-relaxed text-light-subtle dark:text-dark-subtle"
    >
      {{ description }}
    </p>
    <p
      v-if="error"
      :id="`${id}-error`"
      aria-live="polite"
      class="mt-1 font-sans text-xs text-light-love dark:text-dark-love"
    >
      {{ error }}
    </p>
  </div>
</template>
