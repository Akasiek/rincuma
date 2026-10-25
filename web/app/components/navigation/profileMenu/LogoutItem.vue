<script setup lang="ts">
import { DropdownMenuItem } from "reka-ui";

import { confirmNavigationKey } from "~/utils/navigation";

const props = defineProps<{
  menuState: { open: boolean; isLoggingOut: boolean; logoutError: string };
}>();
const confirmNavigation = inject(confirmNavigationKey, undefined);
const { logout } = useAuth();

async function handleLogOut() {
  const state = props.menuState;
  if (state.isLoggingOut) {
    return;
  }

  state.isLoggingOut = true;
  state.logoutError = "";

  try {
    if (confirmNavigation) {
      state.open = false;
      await nextTick();
      if (!(await confirmNavigation())) {
        return;
      }
    }
    await logout();
  } catch {
    state.logoutError = "Could not log out. Please try again.";
    state.open = true;
    return;
  } finally {
    state.isLoggingOut = false;
  }

  state.open = false;
  await navigateTo("/login", { replace: true });
}
</script>

<template>
  <DropdownMenuItem
    as="button"
    type="button"
    :disabled="menuState.isLoggingOut"
    class="flex w-full cursor-pointer items-center gap-2.5 rounded-md px-3 py-2.5 text-left text-light-love outline-none data-disabled:cursor-not-allowed data-disabled:opacity-50 data-highlighted:bg-light-highlight-med dark:text-dark-love dark:data-highlighted:bg-dark-highlight-med"
    @select.prevent="handleLogOut"
  >
    <Icon
      :name="menuState.isLoggingOut ? 'carbon:circle-dash' : 'carbon:logout'"
      :class="{ 'animate-spin motion-reduce:animate-none': menuState.isLoggingOut }"
      class="size-4 shrink-0"
      aria-hidden="true"
    />
    {{ menuState.isLoggingOut ? "Logging out…" : "Log out" }}
  </DropdownMenuItem>
  <p
    v-if="menuState.logoutError"
    role="alert"
    class="px-3 py-2 font-sans text-xs leading-relaxed text-light-love dark:text-dark-love"
  >
    {{ menuState.logoutError }}
  </p>
</template>
