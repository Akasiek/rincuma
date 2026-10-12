<script setup lang="ts">
import type { SelectModelValue } from "../../../shared/types/select";
import type { TaskPriority } from "../../../shared/types/task";

const props = defineProps<{
  id: string;
  compact?: boolean;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();
const model = defineModel<TaskPriority | null>({ default: "none" });

const options = [
  { value: "none", label: "None", iconClass: "text-light-subtle dark:text-dark-subtle" },
  { value: "low", label: "Low", iconClass: "text-light-pine dark:text-dark-foam" },
  { value: "medium", label: "Medium", iconClass: "text-light-gold dark:text-dark-gold" },
  { value: "high", label: "High", iconClass: "text-light-rose dark:text-dark-rose" },
  { value: "urgent", label: "Urgent", iconClass: "text-light-love dark:text-dark-love" },
] satisfies { value: TaskPriority; label: string; iconClass: string }[];

function priorityClass(value: SelectModelValue) {
  return options.find((option) => option.value === value)?.iconClass;
}

const triggerStyle = computed(() =>
  !props.compact || !model.value || model.value === "none"
    ? undefined
    : {
        "--select-trigger-background": "color-mix(in srgb, currentColor 10%, transparent)",
        "--select-trigger-hover-background": "color-mix(in srgb, currentColor 20%, transparent)",
      },
);

function updatePriority(value: SelectModelValue) {
  const option = options.find((option) => option.value === value);
  if (option) {
    model.value = option.value;
  }
}
</script>

<template>
  <AppSelect
    v-bind="props"
    :trigger-class="compact ? priorityClass(model ?? 'none') : undefined"
    :trigger-style="triggerStyle"
    label="Priority"
    :model-value="model ?? 'none'"
    :options="options"
    @update:model-value="updatePriority"
  >
    <template #trigger>
      <span class="relative flex size-5 shrink-0 items-center justify-center" aria-hidden="true">
        <Icon name="carbon:flag" class="relative size-5" />
      </span>
    </template>
    <template #option-icon="{ option }">
      <Icon
        name="carbon:flag"
        :class="priorityClass(option.value)"
        class="mr-2 size-4 shrink-0"
        aria-hidden="true"
      />
    </template>
  </AppSelect>
</template>
