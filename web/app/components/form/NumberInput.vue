<script setup lang="ts">
import {
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
  NumberFieldRoot,
} from "reka-ui";

defineOptions({ inheritAttrs: false });

const model = defineModel<number>();
const { step = 1 } = defineProps<{
  id: string;
  name?: string;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();
</script>

<template>
  <NumberFieldRoot
    :id="id"
    :name="name"
    :model-value="model ?? null"
    :min="min"
    :max="max"
    :step="step"
    :disabled="disabled"
    :format-options="{ useGrouping: false }"
    disable-wheel-change
    :class="
      invalid
        ? 'border-light-love dark:border-dark-love'
        : 'border-light-highlight-med dark:border-dark-highlight-med'
    "
    class="flex items-center overflow-hidden rounded-md border-2 bg-light-base font-sans text-sm dark:bg-dark-base"
    @update:model-value="model = $event"
  >
    <NumberFieldDecrement
      class="flex shrink-0 cursor-pointer items-center justify-center self-stretch p-2.5 hover:bg-light-highlight-med disabled:cursor-not-allowed disabled:opacity-50 dark:hover:bg-dark-highlight-med"
    >
      <Icon name="carbon:subtract" class="size-4" aria-hidden="true" />
    </NumberFieldDecrement>
    <NumberFieldInput
      v-bind="$attrs"
      :aria-invalid="invalid"
      :aria-describedby="describedby"
      class="w-full min-w-0 bg-transparent py-2.5 text-center disabled:cursor-not-allowed disabled:opacity-50"
    />
    <NumberFieldIncrement
      class="flex shrink-0 cursor-pointer items-center justify-center self-stretch p-2.5 hover:bg-light-highlight-med disabled:cursor-not-allowed disabled:opacity-50 dark:hover:bg-dark-highlight-med"
    >
      <Icon name="carbon:add" class="size-4" aria-hidden="true" />
    </NumberFieldIncrement>
  </NumberFieldRoot>
</template>
