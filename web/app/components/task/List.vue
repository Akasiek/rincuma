<script setup lang="ts">
import { groupTasks } from "~/utils/taskList";
import { resolveTaskFilters } from "~/utils/taskFilters";
import type { TaskFilters } from "#shared/types/taskFilters";
import Header from "./list/Header.vue";
import State from "./list/State.vue";
import Section from "./list/Section.vue";

const props = withDefaults(defineProps<{ title?: string; filters?: TaskFilters }>(), {
  title: "All tasks",
  filters: () => ({}),
});

const { listTasks, getTaskListKey, refreshTaskLists } = useTask();
const { now, timeZone } = useFormattedDateTime(null);

const query = computed(() => resolveTaskFilters(props.filters, now.value, timeZone.value));
const taskListKey = computed(() => getTaskListKey(query.value));
const { data, status, error, refresh } = await useAsyncData(taskListKey, () =>
  listTasks(query.value),
);

const tasks = computed(() => data.value ?? []);
const hasLoadedTasks = computed(() => data.value !== undefined);
const collapsedGroups = ref<string[]>(["completed"]);
const groups = computed(() => groupTasks(tasks.value, now.value, timeZone.value));
const activeCount = computed(() => tasks.value.filter((task) => !task.completed_at).length);
const completedCount = computed(() => tasks.value.length - activeCount.value);
const refreshing = computed(() => status.value === "pending");

async function updateTask(updatedTask: Task) {
  data.value = tasks.value
    .map((task) => (task.id === updatedTask.id ? updatedTask : task))
    .filter(
      (task) =>
        !updatedTask.completed_at ||
        !updatedTask.series_id ||
        !task.completed_at ||
        task.series_id !== updatedTask.series_id ||
        task.id === updatedTask.id,
    )
    .filter((task) => {
      const status = query.value.status ?? "all";
      if (status === "all") {
        return true;
      }

      return !!task.completed_at === (status === "completed");
    });

  await refreshTaskLists();
}

function toggleGroup(id: string) {
  if (collapsedGroups.value.includes(id)) {
    collapsedGroups.value = collapsedGroups.value.filter((groupId) => groupId !== id);
  } else {
    collapsedGroups.value.push(id);
  }
}
</script>

<template>
  <main class="min-h-0 min-w-0 flex-1 overflow-hidden" :aria-busy="refreshing">
    <AppScrollArea class="h-full" :label="title">
      <div class="px-4 py-8 pb-28 sm:px-8 lg:px-12 lg:py-12 lg:pb-28">
        <div class="mx-auto max-w-4xl">
          <Header
            :title="title"
            :total="tasks.length"
            :active-count="activeCount"
            :completed-count="completedCount"
            :loaded="hasLoadedTasks"
          />
          <State :failed="!!error" :empty="hasLoadedTasks && tasks.length === 0" @retry="refresh()">
            <div class="space-y-8">
              <Section
                v-for="group in groups"
                :key="group.id"
                :group="group"
                :collapsed="collapsedGroups.includes(group.id)"
                @update:collapsed="toggleGroup(group.id)"
                @updated="updateTask"
              />
            </div>
          </State>
        </div>
      </div>
    </AppScrollArea>
  </main>
</template>
