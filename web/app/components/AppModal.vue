<script setup lang="ts">
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "reka-ui";

const open = defineModel<boolean>("open", { default: false });

const { size = "lg" } = defineProps<{
  title: string;
  description?: string;
  size?: "sm" | "lg";
}>();

const widthClass = computed(() => (size === "sm" ? "max-w-sm" : "max-w-lg"));
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogTrigger v-if="$slots.trigger" as-child>
      <slot name="trigger" />
    </DialogTrigger>

    <DialogPortal>
      <DialogOverlay class="modal-overlay fixed inset-0 z-40 bg-black/40" />

      <DialogContent
        :aria-describedby="description ? undefined : null"
        :class="widthClass"
        class="modal-content fixed top-1/2 left-1/2 z-50 max-h-[calc(100dvh-2rem)] w-[calc(100%-2rem)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-md bg-light-surface font-serif text-light-text dark:bg-dark-surface dark:text-dark-text"
      >
        <DialogTitle as="span" class="sr-only">
          {{ title }}
        </DialogTitle>

        <DialogDescription v-if="description" class="sr-only">
          {{ description }}
        </DialogDescription>

        <slot />
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.modal-overlay[data-state="open"] {
  animation: backdrop-in 180ms ease-out;
}

.modal-overlay[data-state="closed"] {
  animation: backdrop-out 140ms ease-in forwards;
}

.modal-content[data-state="open"] {
  animation: modal-in 180ms ease-out;
}

.modal-content[data-state="closed"] {
  animation: modal-out 140ms ease-in forwards;
}

@keyframes backdrop-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes backdrop-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

@keyframes modal-out {
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
  .modal-overlay,
  .modal-content {
    animation: none !important;
  }
}
</style>
