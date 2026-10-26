<script setup lang="ts">
import {
  ToastAction,
  ToastClose,
  ToastDescription,
  ToastRoot,
  ToastTitle,
  ToastViewport,
  useToastManager,
} from "reka-ui";

const { toasts, close } = useToastManager();
const userStore = useUserStore();
watch(
  () => userStore.user?.id,
  () => close(),
);
</script>

<template>
  <ToastRoot
    v-for="toast in toasts"
    :key="toast.id"
    :toast="toast"
    v-slot="{ remaining, duration }"
    class="app-toast pointer-events-auto relative flex items-center gap-3 overflow-hidden rounded-md border-2 border-light-highlight-med bg-light-surface p-4 font-sans text-sm text-light-text shadow-lg data-[status=error]:border-light-love dark:border-dark-highlight-med dark:bg-dark-surface dark:text-dark-text dark:data-[status=error]:border-dark-love"
  >
    <div class="min-w-0 flex-1">
      <ToastTitle class="font-medium" />
      <ToastDescription class="mt-0.5 wrap-anywhere text-light-subtle dark:text-dark-subtle" />
    </div>
    <ToastAction
      v-if="toast.actionProps"
      class="shrink-0 cursor-pointer rounded-sm px-2 py-1 font-medium text-light-pine hover:bg-light-pine/10 dark:text-dark-pine dark:hover:bg-dark-pine/10"
    />
    <ToastClose
      aria-label="Dismiss notification"
      class="flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-sm hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med"
    >
      <Icon name="carbon:close" class="size-4" aria-hidden="true" />
    </ToastClose>
    <div
      v-if="Number.isFinite(duration) && duration > 0"
      aria-hidden="true"
      class="absolute inset-x-0 bottom-0 h-0.5 bg-light-highlight-med dark:bg-dark-highlight-med"
    >
      <div
        :style="{ transform: `scaleX(${Math.max(0, Math.min(1, remaining / duration))})` }"
        :class="
          toast.status === 'error'
            ? 'bg-light-love dark:bg-dark-love'
            : 'bg-light-pine dark:bg-dark-pine'
        "
        class="h-full origin-left"
      />
    </div>
  </ToastRoot>
  <ToastViewport
    class="pointer-events-none fixed top-4 right-4 z-70 flex max-h-[calc(100dvh-2rem)] w-[calc(100%-2rem)] max-w-sm flex-col gap-2 outline-none sm:top-6 sm:right-6"
  />
</template>

<style>
.app-toast[data-state="open"] {
  animation: toast-in 200ms ease-out;
}

.app-toast[data-state="closed"] {
  animation: toast-out 150ms ease-in forwards;
}

.app-toast[data-swipe="move"] {
  animation: none;
  transform: translateX(var(--reka-toast-swipe-move-x));
}

.app-toast[data-swipe="cancel"] {
  animation: none;
  transform: translateX(0);
  transition: transform 150ms ease-out;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(1.5rem);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toast-out {
  from {
    opacity: 1;
    transform: translateX(var(--reka-toast-swipe-end-x, 0px));
  }
  to {
    opacity: 0;
    transform: translateX(calc(var(--reka-toast-swipe-end-x, 0px) + 1.5rem));
  }
}

@media (prefers-reduced-motion: reduce) {
  .app-toast[data-state] {
    animation: none;
    transition: none;
  }
}
</style>
