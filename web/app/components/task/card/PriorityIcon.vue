<script setup lang="ts">
import type { TaskPriority } from "#shared/types/task";
import { taskPriorities } from "~/utils/taskPriorities.ts";

const props = defineProps<{ priority: TaskPriority }>();

const priorityRecord = computed(() => taskPriorities[props.priority]);
</script>

<template>
  <div
    class="flex items-center"
    :class="priorityRecord.color"
    :title="`${priorityRecord.label} priority`"
    :aria-label="`${priorityRecord.label} priority`"
  >
    <Icon name="carbon:flag" class="size-3.5" aria-hidden="true" />
  </div>
</template>
