<script setup lang="ts">
const { type = "button" } = defineProps<{ type?: "button" | "submit" | "reset" }>();

const slots = useSlots();
const slot = computed(() => slots.default?.().at(0)?.children);
</script>

<template>
  <button
    :label="slot"
    :type="type"
    class="cursor-pointer rounded-md border-2 border-light-highlight-med px-4 py-1.5 font-serif transition-colors hover:bg-light-highlight-med focus:bg-light-highlight-med dark:border-dark-highlight-med hover:dark:bg-dark-highlight-med focus:dark:bg-dark-highlight-med"
  >
    <slot />
  </button>
</template>
