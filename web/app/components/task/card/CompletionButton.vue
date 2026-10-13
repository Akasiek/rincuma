<script setup lang="ts">
import type { TaskPriority } from "#shared/types/task.ts";
import { taskPriorities } from "~/utils/taskPriorities.ts";

const props = withDefaults(
  defineProps<{ completed: boolean; saving: boolean; priority?: TaskPriority }>(),
  { priority: "none" },
);
const emit = defineEmits<{ toggle: [] }>();
const previewBlocked = ref(false);

function toggle() {
  previewBlocked.value = true;
  emit("toggle");
}

function enablePreview() {
  if (!props.saving) {
    previewBlocked.value = false;
  }
}
</script>

<template>
  <button
    type="button"
    class="completion-toggle flex size-7 cursor-pointer items-center justify-center rounded-md disabled:cursor-wait disabled:opacity-60"
    :class="[
      completed ? 'text-light-muted dark:text-dark-muted' : taskPriorities[priority].color,
      {
        'is-completed': completed,
        'is-active': !completed,
        'is-saving': saving,
        'preview-blocked': previewBlocked,
      },
    ]"
    :disabled="saving"
    :aria-pressed="completed"
    :aria-busy="saving"
    :aria-label="completed ? 'Reopen task' : 'Complete task'"
    :title="completed ? 'Reopen task' : 'Complete task'"
    @click="toggle"
    @pointerenter="enablePreview"
    @focus="enablePreview"
  >
    <span
      class="completion-square flex size-4 items-center justify-center rounded-[5px] border-[1.5px] border-current"
    >
      <svg viewBox="0 0 24 24" class="size-3" fill="none" aria-hidden="true">
        <path
          class="completion-mark"
          d="m5 12 4 4L19 6"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </span>
  </button>
</template>

<style scoped>
.completion-square {
  background: color-mix(in srgb, currentColor 6%, transparent);
  transition:
    transform 150ms ease,
    background-color 150ms ease;
}

.completion-toggle:is(:hover, :focus-visible) .completion-square,
.is-completed .completion-square {
  background: color-mix(in srgb, currentColor 16%, transparent);
}

.completion-toggle:active:not(:disabled) .completion-square {
  transform: scale(0.9);
}

.completion-mark {
  stroke-dasharray: 24;
  stroke-dashoffset: 24;
  transition: stroke-dashoffset 200ms ease;
}

.is-completed .completion-mark,
.is-active:not(:disabled, .preview-blocked):is(:hover, :focus-visible, :active) .completion-mark,
.is-active.is-saving .completion-mark {
  stroke-dashoffset: 0;
}

.is-completed:not(:disabled, .preview-blocked):is(:hover, :focus-visible, :active) .completion-mark,
.is-completed.is-saving .completion-mark {
  stroke-dashoffset: 24;
}

@media (prefers-reduced-motion: reduce) {
  .completion-square,
  .completion-mark {
    transition: none;
  }
}
</style>
