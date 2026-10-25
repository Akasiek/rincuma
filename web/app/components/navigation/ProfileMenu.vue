<script setup lang="ts">
import {
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "reka-ui";
const userStore = useUserStore();
const menuState = reactive({ open: false, isLoggingOut: false, logoutError: "" });
const initial = computed(() => userStore.user?.email.slice(0, 1).toUpperCase() ?? "?");
</script>

<template>
  <div v-if="userStore.user" class="min-w-0">
    <DropdownMenuRoot v-model:open="menuState.open">
      <DropdownMenuTrigger as-child>
        <button
          type="button"
          aria-label="Profile menu"
          :disabled="menuState.isLoggingOut"
          class="flex w-full min-w-0 cursor-pointer items-center gap-2.5 rounded-md p-2 text-left hover:bg-light-highlight-med disabled:cursor-not-allowed disabled:opacity-50 dark:hover:bg-dark-highlight-med"
        >
          <span
            aria-hidden="true"
            class="flex size-8 shrink-0 items-center justify-center rounded-full bg-light-pine/10 font-sans text-sm font-semibold text-light-pine dark:bg-dark-foam/10 dark:text-dark-foam"
          >
            {{ initial }}
          </span>
          <span class="min-w-0 flex-1">
            <span class="block font-serif text-sm">Profile</span>
            <span class="block truncate font-sans text-xs text-light-subtle dark:text-dark-subtle">
              {{ userStore.user.email }}
            </span>
          </span>
          <Icon name="carbon:chevron-sort" class="size-4 shrink-0" aria-hidden="true" />
        </button>
      </DropdownMenuTrigger>

      <DropdownMenuPortal>
        <DropdownMenuContent
          side="top"
          align="start"
          :side-offset="8"
          class="z-60 w-64 max-w-[calc(100vw-2rem)] rounded-lg border border-light-highlight-med bg-light-surface p-1 font-serif text-sm text-light-text shadow-lg dark:border-dark-highlight-med dark:bg-dark-surface dark:text-dark-text"
        >
          <DropdownMenuLabel class="px-3 py-2">
            <span class="block text-xs text-light-subtle dark:text-dark-subtle">Signed in as</span>
            <span class="mt-1 block truncate font-sans text-xs">{{ userStore.user.email }}</span>
          </DropdownMenuLabel>
          <DropdownMenuSeparator
            class="my-1 h-px bg-light-highlight-med dark:bg-dark-highlight-med"
          />
          <NavigationProfileMenuEditItem :disabled="menuState.isLoggingOut" />
          <NavigationProfileMenuLogoutItem :menu-state="menuState" />
        </DropdownMenuContent>
      </DropdownMenuPortal>
    </DropdownMenuRoot>
  </div>
</template>
