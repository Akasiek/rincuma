<script setup lang="ts">
import type { TaskFilters } from "#shared/types/taskFilters";

const props = defineProps<{ kind: "project" | "tag"; id: string }>();
const store = useDictionariesStore();
const requestFetch = useRequestFetch();
const userStore = useUserStore();
const id = Number(props.id);

if (!/^[1-9]\d*$/.test(props.id) || !Number.isSafeInteger(id)) {
  throw createError({ status: 404, statusText: "Not found", fatal: true });
}

const key = `dictionary-view:${userStore.user?.id}:${props.kind}:${id}`;
const { data: record, error } = await useAsyncData(key, () =>
  requestFetch<Project | Tag>(`/api/${props.kind}s/${id}`),
);

if (error.value) {
  const notFound = error.value.status === 404;
  throw createError({
    status: notFound ? 404 : 503,
    statusText: notFound ? "Not found" : "Could not load this view",
    fatal: true,
  });
}

const current = computed(() => {
  const records = props.kind === "project" ? store.projects : store.tags;
  return records.find((value) => value.id === id);
});
const title = computed(() => current.value?.name ?? record.value?.name ?? "Tasks");
const filters: TaskFilters = props.kind === "project" ? { project_id: id } : { tag_id: id };
useHead({ title });

watch(current, (value, previous) => {
  if (previous && !value) {
    showError({ status: 404, statusText: "Not found" });
  }
});
</script>

<template>
  <TaskWorkspace>
    <TaskList :title="title" :filters="filters" />
  </TaskWorkspace>
</template>
