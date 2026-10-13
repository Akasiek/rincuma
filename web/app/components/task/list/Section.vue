<script setup lang="ts">
import type { Task } from "#shared/types/task";
import type { TaskGroup } from "~/utils/taskList";

defineProps<{ group: TaskGroup }>();

const collapsed = defineModel<boolean>("collapsed", { default: false });
const emit = defineEmits<{ updated: [task: Task] }>();
</script>

<template>
  <section :aria-label="group.label">
    <h2>
      <button
        type="button"
        class="mb-1 flex w-full cursor-pointer items-center gap-2 rounded-md py-2 text-left font-serif text-sm font-semibold"
        :class="{
          'text-light-love dark:text-dark-love': group.id === 'overdue',
          'text-light-subtle dark:text-dark-subtle': group.id === 'completed',
        }"
        :aria-expanded="!collapsed"
        :aria-controls="`${group.id}-tasks`"
        @click="collapsed = !collapsed"
      >
        <Icon
          :name="collapsed ? 'carbon:chevron-right' : 'carbon:chevron-down'"
          class="size-3.5 shrink-0"
          aria-hidden="true"
        />
        <span>{{ group.label }}</span>
        <span
          class="font-mono text-xs font-normal text-light-subtle tabular-nums dark:text-dark-subtle"
        >
          {{ group.tasks.length }}
        </span>
      </button>
    </h2>
    <ul v-show="!collapsed" :id="`${group.id}-tasks`">
      <li v-for="task in group.tasks" :key="task.id" class="min-w-0">
        <TaskCard :task="task" @updated="emit('updated', $event)" />
      </li>
    </ul>
  </section>
</template>
