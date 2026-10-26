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
    class="pointer-events-auto flex items-center gap-3 rounded-md border-2 border-light-highlight-med bg-light-surface p-4 font-sans text-sm text-light-text shadow-lg data-[status=error]:border-light-love dark:border-dark-highlight-med dark:bg-dark-surface dark:text-dark-text dark:data-[status=error]:border-dark-love"
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
      class="shrink-0 cursor-pointer rounded-sm p-1 hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med"
    >
      <Icon name="carbon:close" class="size-4" aria-hidden="true" />
    </ToastClose>
  </ToastRoot>
  <ToastViewport
    class="pointer-events-none fixed top-4 right-4 z-70 flex max-h-[calc(100dvh-2rem)] w-[calc(100%-2rem)] max-w-sm flex-col gap-2 outline-none sm:top-6 sm:right-6"
  />
</template>
