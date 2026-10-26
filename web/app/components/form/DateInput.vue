<script setup lang="ts">
import { parseDate, type DateValue } from "@internationalized/date";
import { DateFieldInput, DateFieldRoot } from "reka-ui";

defineOptions({ inheritAttrs: false });

const model = defineModel<string | null>();
defineProps<{
  id: string;
  name?: string;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();

const date = computed(() => {
  if (!model.value) {
    return null;
  }
  try {
    return parseDate(model.value);
  } catch {
    return null;
  }
});

function updateDate(value: DateValue | undefined) {
  model.value = value?.toString() ?? "";
}
</script>

<template>
  <DateFieldRoot
    v-bind="$attrs"
    :id="id"
    :name="name"
    :model-value="date"
    :disabled="disabled"
    :aria-invalid="invalid"
    :aria-describedby="describedby"
    v-slot="{ segments }"
    granularity="day"
    locale="en-GB"
    :class="
      invalid
        ? 'border-light-love dark:border-dark-love'
        : 'border-light-highlight-med dark:border-dark-highlight-med'
    "
    class="flex w-full items-center rounded-md border-2 bg-light-base px-3 py-2.5 font-sans text-sm data-disabled:cursor-not-allowed data-disabled:opacity-50 dark:bg-dark-base"
    @update:model-value="updateDate"
  >
    <DateFieldInput
      v-for="segment in segments"
      :key="segment.part"
      :part="segment.part"
      class="rounded-sm px-0.5 tabular-nums focus:bg-light-highlight-med data-placeholder:text-light-muted dark:focus:bg-dark-highlight-med dark:data-placeholder:text-dark-muted"
    >
      {{ segment.value }}
    </DateFieldInput>
  </DateFieldRoot>
</template>
