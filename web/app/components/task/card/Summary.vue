<script setup lang="ts">
import type { Task } from "#shared/types/task";
import TaskCardDueDate from "./DueDate.vue";

defineProps<{ task: Task; descriptionId: string }>();
const descriptionOpen = defineModel<boolean>("descriptionOpen", { default: false });
</script>

<template>
  <div class="min-w-0 flex-1">
    <h3
      class="font-serif text-[0.9em] leading-6 font-normal wrap-anywhere"
      :class="{
        'text-light-subtle line-through decoration-light-muted/50 dark:text-dark-subtle dark:decoration-dark-muted/50':
          task.completed_at,
      }"
    >
      {{ task.name }}
    </h3>
    <div
      v-if="task.due_at || task.description || task.parent_id !== null"
      class="mt-0.5 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-[0.7em] leading-4 text-light-subtle dark:text-dark-subtle"
    >
      <TaskCardDueDate v-if="task.due_at" :due-at="task.due_at" :completed="!!task.completed_at" />

      <span v-if="task.parent_id !== null" class="inline-flex items-center gap-1 font-sans">
        <Icon name="carbon:tree-view-alt" class="size-3" aria-hidden="true" />
        Subtask
      </span>

      <button
        v-if="task.description"
        type="button"
        class="inline-flex cursor-pointer items-center gap-1 rounded-sm font-sans hover:text-light-text dark:hover:text-dark-text"
        :aria-expanded="descriptionOpen"
        :aria-controls="descriptionId"
        @click="descriptionOpen = !descriptionOpen"
      >
        <Icon name="carbon:text-align-left" class="size-3" aria-hidden="true" />
        {{ descriptionOpen ? "Hide details" : "Details" }}
      </button>
    </div>
  </div>
</template>
