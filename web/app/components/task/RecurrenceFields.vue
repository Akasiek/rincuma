<script setup lang="ts">
import type { RecurrenceConfig } from "#shared/types/task";
import type { SelectModelValue } from "#shared/types/select";

const props = defineProps<{
  id: string;
  dueAt?: string | null;
  occurrenceCount?: number;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();
const model = defineModel<RecurrenceConfig | null>();
const { timeZone } = useTimeZone();

const frequencies = [
  { value: "none", label: "Does not repeat" },
  { value: "daily", label: "Days" },
  { value: "weekly", label: "Weeks" },
  { value: "monthly", label: "Months" },
];
const endings = [
  { value: "never", label: "Never" },
  { value: "through", label: "On a date" },
  { value: "after", label: "After occurrences" },
];

function updateFrequency(value: SelectModelValue) {
  if (value === "none") {
    model.value = null;
    return;
  }
  if (value !== "daily" && value !== "weekly" && value !== "monthly") {
    return;
  }

  model.value = {
    interval: 1,
    time_zone: timeZone.value,
    end: { mode: "never" },
    ...model.value,
    frequency: value,
  };
}

function updateRule(patch: Partial<RecurrenceConfig>) {
  if (model.value) {
    model.value = { ...model.value, ...patch };
  }
}

function updateEnd(value: SelectModelValue) {
  if (value === "never") {
    updateRule({ end: { mode: "never" } });
  } else if (value === "through") {
    updateRule({ end: { mode: "through", date: "" } });
  } else if (value === "after") {
    updateRule({ end: { mode: "after", count: Math.max(props.occurrenceCount ?? 1, 2) } });
  }
}
</script>

<template>
  <div class="space-y-3 font-sans text-sm">
    <AppSelect
      :id="id"
      label="Repeat every"
      :model-value="model?.frequency ?? 'none'"
      :options="frequencies"
      :disabled="disabled"
      :invalid="invalid"
      :describedby="describedby"
      @update:model-value="updateFrequency"
    />
    <template v-if="model">
      <div class="grid gap-3 sm:grid-cols-2">
        <div>
          <FormLabel :for="`${id}-interval`">Interval</FormLabel>
          <FormNumberInput
            :id="`${id}-interval`"
            :min="1"
            :max="4294967295"
            :model-value="model.interval"
            :disabled="disabled"
            @update:model-value="updateRule({ interval: $event ?? NaN })"
          />
        </div>
        <div>
          <FormLabel :for="`${id}-zone`">Time zone</FormLabel>
          <FormInput
            :id="`${id}-zone`"
            :model-value="model.time_zone"
            :disabled="disabled"
            @update:model-value="updateRule({ time_zone: $event ?? '' })"
          />
        </div>
      </div>
      <AppSelect
        :id="`${id}-end`"
        label="Ends"
        :model-value="model.end.mode"
        :options="endings"
        :disabled="disabled"
        @update:model-value="updateEnd"
      />
      <div v-if="model.end.mode !== 'never'">
        <FormLabel :id="`${id}-limit-label`" :for="`${id}-limit`">
          {{ model.end.mode === "through" ? "Last date (inclusive)" : "Total occurrences" }}
        </FormLabel>
        <FormNumberInput
          v-if="model.end.mode === 'after'"
          :id="`${id}-limit`"
          :model-value="model.end.count"
          :min="occurrenceCount ?? 1"
          :max="4294967295"
          :disabled="disabled"
          @update:model-value="updateRule({ end: { mode: 'after', count: $event ?? NaN } })"
        />
        <FormDateInput
          v-else
          :id="`${id}-limit`"
          :aria-labelledby="`${id}-limit-label`"
          :model-value="model.end.date"
          :disabled="disabled"
          @update:model-value="updateRule({ end: { mode: 'through', date: $event ?? '' } })"
        />
        <p
          v-if="model.end.mode === 'after'"
          class="mt-1 text-xs text-light-subtle dark:text-dark-subtle"
        >
          Includes occurrences already created: {{ occurrenceCount ?? 1 }}.
        </p>
      </div>
      <p v-if="!dueAt" class="text-xs text-light-subtle dark:text-dark-subtle">
        Set a date and time to repeat this task.
      </p>
    </template>
  </div>
</template>
