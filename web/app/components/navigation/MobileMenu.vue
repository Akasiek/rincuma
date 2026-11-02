<script setup lang="ts">
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "reka-ui";

const open = ref(false);
const route = useRoute();
const desktop = useMediaQuery("(min-width: 768px)");

watch(
  () => route.path,
  () => {
    open.value = false;
  },
);
watch(desktop, (value) => {
  if (value) {
    open.value = false;
  }
});
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogTrigger as-child>
      <Button size="sm">
        <Icon name="carbon:menu" class="size-4" aria-hidden="true" />
        Menu
      </Button>
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="mobile-menu-overlay fixed inset-0 z-50 bg-black/40" />
      <DialogContent
        :aria-describedby="undefined"
        class="mobile-menu-content fixed inset-y-0 left-0 z-50 h-dvh w-[min(20rem,calc(100%-3rem))] overflow-hidden bg-light-surface font-serif text-light-text dark:bg-dark-surface dark:text-dark-text"
      >
        <DialogTitle class="sr-only">Navigation</DialogTitle>
        <NavigationContent @navigate="open = false">
          <template #actions>
            <DialogClose as-child>
              <button
                type="button"
                aria-label="Close menu"
                class="flex size-10 cursor-pointer items-center justify-center rounded-md hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med"
              >
                <Icon name="carbon:close" class="size-5" aria-hidden="true" />
              </button>
            </DialogClose>
          </template>
        </NavigationContent>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.mobile-menu-overlay[data-state="open"] {
  animation: mobile-menu-overlay-in 220ms ease-out;
}

.mobile-menu-overlay[data-state="closed"] {
  animation: mobile-menu-overlay-out 180ms ease-in forwards;
}

.mobile-menu-content[data-state="open"] {
  animation: mobile-menu-content-in 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.mobile-menu-content[data-state="closed"] {
  animation: mobile-menu-content-out 180ms ease-in forwards;
}

@keyframes mobile-menu-overlay-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes mobile-menu-overlay-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

@keyframes mobile-menu-content-in {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(0);
  }
}

@keyframes mobile-menu-content-out {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(-100%);
  }
}

@media (prefers-reduced-motion: reduce) {
  .mobile-menu-overlay,
  .mobile-menu-content {
    animation: none !important;
  }
}
</style>
