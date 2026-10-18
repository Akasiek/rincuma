<script setup lang="ts">
import { Label } from "reka-ui";

const props = defineProps<{ for: string; optional?: boolean }>();
</script>

<template>
  <Label :for="props.for" class="font-serif text-sm font-medium">
    <slot />
    <span v-if="optional" class="font-normal text-light-subtle dark:text-dark-subtle">
      (optional)
    </span>
  </Label>
</template>
