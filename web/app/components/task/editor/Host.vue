<script setup lang="ts">
import { taskIdFromQuery } from "~/utils/taskEditor";

const route = useRoute();
const router = useRouter();
const open = computed(() => route.query.task !== undefined);
const taskId = computed(() => taskIdFromQuery(route.query.task));
const backgroundLocation = ref<string | null>(null);

const removeHook = router.afterEach((to, from, failure) => {
  if (failure) {
    return;
  }

  const openedFromList =
    to.query.task !== undefined &&
    from.query.task === undefined &&
    to.path === from.path &&
    from.matched.length > 0;
  backgroundLocation.value = openedFromList ? from.fullPath : null;
});
onScopeDispose(removeHook);

function close() {
  if (backgroundLocation.value && router.options.history.state.back === backgroundLocation.value) {
    router.back();
    return;
  }

  const query = { ...route.query };
  delete query.task;
  void router.replace({ path: route.path, query, hash: route.hash });
}
</script>

<template>
  <TaskEditor v-if="open" :key="String(route.query.task)" :task-id="taskId" @close="close" />
</template>
