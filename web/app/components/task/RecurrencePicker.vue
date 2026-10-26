<script setup lang="ts">
import type { RecurrenceConfig } from "#shared/types/task";
import { validateRecurrenceConfig } from "~/utils/taskRecurrence";

import { cloneRecurrenceConfig, recurrenceSummary } from "~/utils/taskRecurrencePattern";

const props = defineProps<{
  id: string;
  dueAt?: string | null;
  anchor?: string;
  occurrenceCount?: number;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();
const model = defineModel<RecurrenceConfig | null>();
const open = ref(false);
const draft = ref<RecurrenceConfig | null>();
const error = ref("");
const errorId = `${props.id}-error`;
const summary = computed(() => recurrenceSummary(model.value));

watch(open, (isOpen) => {
  if (isOpen) {
    const rule = model.value;
    draft.value = rule ? cloneRecurrenceConfig(rule) : rule;
    error.value = "";
  }
});

function apply() {
  const result = validateRecurrenceConfig(draft.value, props.occurrenceCount);
  if (result !== true) {
    error.value = result;
    return;
  }
  model.value = draft.value;
  open.value = false;
}
</script>

<template>
  <AppModal v-model:open="open" title="Repeat task" size="sm">
    <template #trigger>
      <button
        :id="id"
        type="button"
        :disabled="disabled"
        :aria-invalid="invalid"
        :aria-describedby="describedby"
        :aria-label="`Repeat: ${summary}`"
        :title="`Repeat: ${summary}`"
        :class="
          model
            ? 'bg-light-pine/10 text-light-pine hover:bg-light-pine/20 dark:bg-dark-pine/10 dark:text-dark-pine dark:hover:bg-dark-pine/20'
            : 'hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med'
        "
        class="flex shrink-0 cursor-pointer items-center justify-center rounded-md border-2 border-transparent px-2.5 py-1.5 disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Icon name="carbon:renew" class="size-5 shrink-0" aria-hidden="true" />
      </button>
    </template>

    <div class="space-y-4 p-4 sm:p-6">
      <TaskRecurrenceFields
        v-model="draft"
        :id="`${id}-draft`"
        :due-at="dueAt"
        :anchor="anchor"
        :occurrence-count="occurrenceCount"
        :invalid="!!error"
        :describedby="error ? errorId : undefined"
      />
      <p
        v-if="error"
        :id="errorId"
        role="alert"
        class="font-sans text-sm text-light-love dark:text-dark-love"
      >
        {{ error }}
      </p>
      <div class="flex justify-end gap-2">
        <Button size="sm" @click="open = false">Cancel</Button>
        <Button size="sm" @click="apply">Apply</Button>
      </div>
    </div>
  </AppModal>
</template>
