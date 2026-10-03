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

defineOptions({ inheritAttrs: false });

const open = defineModel<boolean>("open", { default: false });

defineProps<{
  title: string;
  description?: string;
}>();
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
        v-bind="$attrs"
        class="fixed top-1/2 left-1/2 z-50 -translate-x-1/2 -translate-y-1/2"
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
@keyframes backdrop-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
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
@media (prefers-reduced-motion: reduce) {
  .modal-overlay {
    animation: none !important;
  }
}
</style>
