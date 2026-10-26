<script setup lang="ts">
import { ToggleGroupItem, ToggleGroupRoot } from "reka-ui";
import type { RecurrenceConfig } from "#shared/types/task";
import type { SelectModelValue } from "#shared/types/select";
import {
  recurrenceOrdinals,
  recurrenceWeekdays,
  recurrenceReference,
} from "~/utils/taskRecurrencePattern";

const props = defineProps<{
  id: string;
  dueAt?: string | null;
  anchor?: string;
  disabled?: boolean;
}>();
const model = defineModel<RecurrenceConfig>({ required: true });
const reference = computed(() =>
  recurrenceReference(props.dueAt, model.value.time_zone, props.anchor),
);
const selectedDays = computed(() =>
  model.value.pattern?.kind === "weekdays" ? model.value.pattern.days : [reference.value.weekday],
);
const monthlyPattern = computed(() =>
  model.value.pattern?.kind === "monthly_weekday" ? model.value.pattern : undefined,
);

function updateDays(value: unknown) {
  if (Array.isArray(value)) {
    model.value = {
      ...model.value,
      pattern: {
        kind: "weekdays",
        days: value
          .filter((day): day is number => typeof day === "number")
          .sort((left, right) => left - right),
      },
    };
  }
}

function updateMonthlyMode(value: SelectModelValue) {
  if (value === "date") {
    model.value = { ...model.value, pattern: undefined };
  } else if (value === "weekday") {
    const reference = recurrenceReference(props.dueAt, model.value.time_zone);
    model.value = {
      ...model.value,
      pattern: {
        kind: "monthly_weekday",
        ordinal: reference.ordinal,
        weekday: reference.weekday,
      },
    };
  }
}

function updateMonthlyPart(part: "ordinal" | "weekday", value: SelectModelValue) {
  if (monthlyPattern.value && typeof value === "number") {
    model.value = { ...model.value, pattern: { ...monthlyPattern.value, [part]: value } };
  }
}
</script>

<template>
  <div v-if="model.frequency === 'weekly'" class="space-y-1.5">
    <p :id="`${id}-days-label`" class="text-xs text-light-subtle dark:text-dark-subtle">
      Repeat on
    </p>
    <ToggleGroupRoot
      type="multiple"
      :model-value="selectedDays"
      :disabled="disabled"
      :aria-labelledby="`${id}-days-label`"
      class="grid grid-cols-7 gap-1"
      @update:model-value="updateDays"
    >
      <ToggleGroupItem
        v-for="day in recurrenceWeekdays"
        :key="day.value"
        :value="day.value"
        :aria-label="day.label"
        :title="day.label"
        class="cursor-pointer rounded-md border-2 border-light-highlight-med py-2 text-center text-xs disabled:cursor-not-allowed disabled:opacity-50 data-[state=on]:border-light-pine data-[state=on]:bg-light-pine/10 data-[state=on]:text-light-pine dark:border-dark-highlight-med dark:data-[state=on]:border-dark-pine dark:data-[state=on]:bg-dark-pine/10 dark:data-[state=on]:text-dark-pine"
      >
        {{ day.label.slice(0, 2) }}
      </ToggleGroupItem>
    </ToggleGroupRoot>
    <p v-if="model.interval > 1" class="text-xs text-light-subtle dark:text-dark-subtle">
      Weeks start on Monday. The first due date sets the starting week.
    </p>
  </div>
  <div v-else-if="model.frequency === 'monthly'" class="space-y-3">
    <AppSelect
      :id="`${id}-monthly-mode`"
      label="Repeat on"
      :model-value="monthlyPattern ? 'weekday' : 'date'"
      :options="[
        { value: 'date', label: 'Day of the month' },
        { value: 'weekday', label: 'Weekday of the month' },
      ]"
      :disabled="disabled"
      @update:model-value="updateMonthlyMode"
    />
    <div v-if="monthlyPattern" class="grid grid-cols-2 gap-3">
      <AppSelect
        :id="`${id}-ordinal`"
        label="Which"
        :model-value="monthlyPattern.ordinal"
        :options="recurrenceOrdinals"
        :disabled="disabled"
        @update:model-value="updateMonthlyPart('ordinal', $event)"
      />
      <AppSelect
        :id="`${id}-weekday`"
        label="Weekday"
        :model-value="monthlyPattern.weekday"
        :options="recurrenceWeekdays"
        :disabled="disabled"
        @update:model-value="updateMonthlyPart('weekday', $event)"
      />
    </div>
    <p v-if="monthlyPattern?.ordinal === 5" class="text-xs text-light-subtle dark:text-dark-subtle">
      Months without a fifth occurrence of this weekday are skipped.
    </p>
  </div>
  <p v-if="model.frequency !== 'daily'" class="text-xs text-light-subtle dark:text-dark-subtle">
    These days apply to the next occurrences. The current due date stays unchanged.
  </p>
</template>
