<script setup lang="ts">
import BaseModal from "./BaseModal.vue";

const open = defineModel<boolean>("open", { default: false });
const { size = "lg" } = defineProps<{
  title: string;
  description?: string;
  size?: "sm" | "lg";
}>();
const widthClass = computed(() => (size === "sm" ? "max-w-sm" : "max-w-lg"));
</script>

<template>
  <BaseModal
    v-model:open="open"
    :title="title"
    :description="description"
    :class="widthClass"
    class="app-modal-content max-h-[calc(100dvh-2rem)] w-[calc(100%-2rem)] overflow-y-auto rounded-md bg-light-surface font-serif text-light-text dark:bg-dark-surface dark:text-dark-text"
  >
    <template v-if="$slots.trigger" #trigger>
      <slot name="trigger" />
    </template>
    <slot />
  </BaseModal>
</template>

<style>
.app-modal-content[data-state="open"] {
  animation: app-modal-in 180ms ease-out;
}

.app-modal-content[data-state="closed"] {
  animation: app-modal-out 140ms ease-in forwards;
}

@keyframes app-modal-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes app-modal-out {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(8px) scale(0.97);
  }
}

@media (prefers-reduced-motion: reduce) {
  .app-modal-content {
    animation: none !important;
  }
}
</style>
