<script setup lang="ts">
import { DropdownMenuItem } from "reka-ui";

defineProps<{ disabled: boolean }>();
</script>

<template>
  <DropdownMenuItem as-child :disabled="disabled">
    <NuxtLink
      to="/profile/edit"
      class="flex cursor-pointer items-center gap-2.5 rounded-md px-3 py-2.5 outline-none data-disabled:pointer-events-none data-disabled:opacity-50 data-highlighted:bg-light-highlight-med dark:data-highlighted:bg-dark-highlight-med"
    >
      <Icon name="carbon:user-profile" class="size-4 shrink-0" aria-hidden="true" />
      Edit profile
    </NuxtLink>
  </DropdownMenuItem>
</template>
