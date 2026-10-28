<script setup lang="ts">
import type { SelectModelValue } from "#shared/types/select.ts";
import { storeToRefs } from "pinia";

const props = defineProps<{
  id: string;
  compact?: boolean;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();
const creating = ref(false);

const model = defineModel<number[]>({ default: () => [] });
const { tags, loaded } = storeToRefs(useDictionariesStore());

const options = computed(() => tags.value.map((tag) => ({ value: tag.id, label: tag.name })));

const selectedColor = computed(() =>
  model.value.length !== 1 ? undefined : tagColor(model.value[0]),
);

const toneClass = computed(() =>
  !model.value.length
    ? "text-light-subtle dark:text-dark-subtle"
    : "text-light-foam dark:text-dark-foam",
);

const triggerStyle = computed(() =>
  !props.compact || !model.value.length
    ? undefined
    : {
        color: selectedColor.value || undefined,
        "--select-trigger-background": "color-mix(in srgb, currentColor 10%, transparent)",
        "--select-trigger-hover-background": "color-mix(in srgb, currentColor 20%, transparent)",
      },
);

function updateTags(value: SelectModelValue) {
  if (!Array.isArray(value)) {
    return;
  }

  model.value = value.filter((id): id is number => typeof id === "number");
}

function tagColor(value: SelectModelValue | undefined) {
  return tags.value.find((tag) => tag.id === value)?.color ?? undefined;
}
function focusTrigger(event: Event) {
  event.preventDefault();
  document.getElementById(props.id)?.focus();
}
</script>

<template>
  <div class="min-w-0">
    <AppSelect
      v-bind="props"
      :trigger-class="compact ? toneClass : undefined"
      :trigger-style="triggerStyle"
      label="Tags"
      create-label="Create tag…"
      @create="creating = true"
      :model-value="model"
      :options="options"
      multiple
      :placeholder="tags.length ? 'Select tags' : 'No tags available'"
      :disabled="disabled || !loaded"
      @update:model-value="updateTags"
    >
      <template #trigger>
        <span class="relative flex size-5 shrink-0 items-center justify-center" aria-hidden="true">
          <Icon name="carbon:tag" class="relative size-5" />
        </span>
        <span
          v-if="model.length"
          :class="toneClass"
          class="selection-count pointer-events-none absolute top-0 right-0 flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] leading-none font-bold tabular-nums"
        >
          {{ model.length }}
        </span>
      </template>
      <template #option-icon="{ option }">
        <span
          v-if="tagColor(option.value)"
          :style="{ backgroundColor: tagColor(option.value) }"
          class="mr-2 size-2.5 shrink-0 rounded-full ring-1 ring-black/10 dark:ring-white/20"
          aria-hidden="true"
        />
      </template>
    </AppSelect>
    <TagCreator
      v-model:open="creating"
      @created="model = [...model, $event.id]"
      @close-auto-focus="focusTrigger"
    />
  </div>
</template>

<style scoped>
.selection-count {
  background-color: color-mix(in srgb, currentColor 18%, var(--color-light-surface));
}

@media (prefers-color-scheme: dark) {
  .selection-count {
    background-color: color-mix(in srgb, currentColor 18%, var(--color-dark-surface));
  }
}
</style>
