<script setup lang="ts">
import type { CSSProperties } from "vue";
import type { SelectModelValue, SelectOption, SelectOptionValue } from "#shared/types/select.ts";
import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from "reka-ui";

const props = defineProps<{
  id: string;
  label: string;
  modelValue: SelectModelValue;
  options: SelectOption[];
  multiple?: boolean;
  placeholder?: string;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
  compact?: boolean;
  triggerClass?: string;
  triggerStyle?: CSSProperties;
}>();

const emit = defineEmits<{ "update:modelValue": [value: SelectModelValue] }>();

const selectedOptions = computed(() => {
  const values = Array.isArray(props.modelValue) ? props.modelValue : [props.modelValue];
  return props.options.filter((option) => values.includes(option.value));
});

const selectedLabels = computed(() =>
  selectedOptions.value.map((option) => option.label).join(", "),
);

function updateValue(value: SelectOptionValue | SelectOptionValue[]) {
  emit(
    "update:modelValue",
    Array.isArray(value) ? value.filter((item): item is string | number => item !== null) : value,
  );
}
</script>

<template>
  <div class="min-w-0">
    <label
      :for="id"
      :class="
        compact
          ? 'sr-only'
          : 'mb-1.5 block font-sans text-xs text-light-subtle dark:text-dark-subtle'
      "
    >
      {{ label }}
    </label>
    <SelectRoot
      :model-value="modelValue"
      :multiple="multiple"
      :disabled="disabled"
      @update:model-value="updateValue"
    >
      <SelectTrigger
        :id="id"
        :aria-invalid="invalid"
        :aria-describedby="describedby"
        :aria-label="compact ? `${label}: ${selectedLabels || placeholder || 'None'}` : undefined"
        :title="`${label}: ${selectedLabels || placeholder}`"
        :style="triggerStyle"
        :class="[
          compact
            ? 'compact-trigger border-transparent px-2.5 py-1.5'
            : 'w-full justify-between border-light-highlight-med px-3 py-2 hover:bg-light-highlight-med dark:border-dark-highlight-med dark:hover:bg-dark-highlight-med',
          triggerClass,
        ]"
        class="relative flex max-w-full min-w-0 cursor-pointer items-center gap-2 rounded-md border-2 text-left font-sans text-sm transition-colors focus:border-light-text focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-light-text disabled:cursor-not-allowed disabled:opacity-50 dark:focus:border-dark-text dark:focus-visible:outline-dark-text"
      >
        <slot v-if="compact" name="trigger">
          {{ selectedLabels || placeholder }}
        </slot>
        <span v-if="!compact" class="min-w-0 flex-1 truncate">
          <SelectValue :placeholder="placeholder">{{ selectedLabels || placeholder }}</SelectValue>
        </span>
        <Icon
          v-if="!compact"
          name="carbon:chevron-down"
          class="size-4 shrink-0"
          aria-hidden="true"
        />
      </SelectTrigger>

      <SelectPortal>
        <SelectContent
          position="popper"
          align="start"
          :side-offset="6"
          class="app-select-content z-60 max-h-(--reka-select-content-available-height) w-(--reka-select-trigger-width) min-w-42 overflow-hidden rounded-md border-2 border-light-highlight-med bg-light-surface font-sans text-sm text-light-text shadow-lg dark:border-dark-highlight-med dark:bg-dark-surface dark:text-dark-text"
        >
          <SelectViewport class="max-h-64 p-1">
            <SelectItem
              v-for="option in options"
              :key="option.value ?? 'none'"
              :value="option.value"
              class="relative flex cursor-pointer items-center rounded-sm py-2 pr-8 pl-2 wrap-anywhere outline-none select-none data-highlighted:bg-light-highlight-med dark:data-highlighted:bg-dark-highlight-med"
            >
              <slot name="option-icon" :option="option" />
              <SelectItemText>{{ option.label }}</SelectItemText>
              <SelectItemIndicator class="absolute right-2 flex items-center">
                <Icon name="carbon:checkmark" class="size-4" aria-hidden="true" />
              </SelectItemIndicator>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>
  </div>
</template>

<style>
/* SelectContent is portaled through fragment roots, so its motion styles must be global. */
.app-select-content {
  --select-offset-x: 0px;
  --select-offset-y: -6px;
  transform-origin: var(--reka-select-content-transform-origin);
}

.app-select-content[data-side="top"] {
  --select-offset-y: 6px;
}

.app-select-content[data-side="left"] {
  --select-offset-x: 6px;
  --select-offset-y: 0px;
}

.app-select-content[data-side="right"] {
  --select-offset-x: -6px;
  --select-offset-y: 0px;
}

.app-select-content[data-state="open"] {
  animation: app-select-in 220ms ease-out;
}

.app-select-content[data-state="closed"] {
  pointer-events: none;
  animation: app-select-out 140ms ease-in forwards;
}

@keyframes app-select-in {
  from {
    opacity: 0;
    transform: translate(var(--select-offset-x), var(--select-offset-y)) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translate(0, 0) scale(1);
  }
}

@keyframes app-select-out {
  from {
    opacity: 1;
    transform: translate(0, 0) scale(1);
  }
  to {
    opacity: 0;
    transform: translate(var(--select-offset-x), var(--select-offset-y)) scale(0.98);
  }
}

@media (prefers-reduced-motion: reduce) {
  .app-select-content {
    animation: none !important;
  }
}
</style>

<style scoped>
.compact-trigger {
  background-color: var(--select-trigger-background, transparent);
}

.compact-trigger:hover {
  background-color: var(--select-trigger-hover-background, var(--color-light-highlight-med));
}

@media (prefers-color-scheme: dark) {
  .compact-trigger:hover {
    background-color: var(--select-trigger-hover-background, var(--color-dark-highlight-med));
  }
}
</style>
