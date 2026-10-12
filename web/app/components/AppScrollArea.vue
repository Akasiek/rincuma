<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaViewport } from "reka-ui";

defineProps<{ label: string }>();
</script>

<template>
  <ScrollAreaRoot type="auto" class="relative min-h-0 min-w-0 overflow-hidden">
    <ScrollAreaViewport
      :aria-label="label"
      role="region"
      class="size-full overscroll-contain focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-light-pine dark:focus-visible:outline-dark-foam"
    >
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar orientation="vertical" class="flex w-2.5 touch-none p-0.5 select-none">
      <ScrollAreaThumb
        class="relative min-h-8 flex-1 rounded-full bg-light-muted/40 hover:bg-light-muted/70 dark:bg-dark-muted/40 dark:hover:bg-dark-muted/70"
      />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>
