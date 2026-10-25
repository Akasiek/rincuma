<script setup lang="ts">
import type { Task, SaveTaskRequest } from "#shared/types/task";
import TaskForm from "./Form.vue";
import TaskModal from "./Modal.vue";

const props = defineProps<{ taskId: number | null }>();
const emit = defineEmits<{ close: []; saved: [] }>();
const { getTask, updateTask, refreshTaskLists } = useTask();
const task = shallowRef<Task | null>(null);
const loading = ref(false);
const loadError = ref("");
const saveError = ref("");
const validationErrors = ref<{ fieldId?: string; message: string }[]>([]);
const isSubmitting = ref(false);
const dirty = ref(false);
const saved = ref(false);
const formId = useId();
const errorSummaryId = `${formId}-errors`;
let disposed = false;

useUnsavedChanges({
  dirty: computed(() => dirty.value && !saved.value),
  saving: computed(() => isSubmitting.value && !saved.value),
  shouldConfirm: (to, from) => to.path !== from.path || to.query.task !== from.query.task,
});
const errors = computed(() => {
  const messages = [...validationErrors.value];
  if (saveError.value) {
    messages.push({ message: saveError.value });
  }
  return messages;
});

async function loadTask() {
  if (loading.value) {
    return;
  }
  if (props.taskId === null) {
    loadError.value = "This task link is invalid.";
    return;
  }

  loading.value = true;
  loadError.value = "";

  try {
    const loaded = await getTask(props.taskId);
    if (disposed) {
      return;
    }

    task.value = loaded;
  } catch (error) {
    if (disposed) {
      return;
    }

    const status = (error as { response?: { status?: number } })?.response?.status;
    loadError.value =
      status === 404
        ? "This task is no longer available."
        : "Could not load this task. Please try again.";
  } finally {
    loading.value = false;
  }
}

async function saveTask(values: SaveTaskRequest) {
  if (!task.value) {
    return;
  }

  try {
    await updateTask(task.value.id, { ...values, parent_id: task.value.parent_id });
  } catch (error) {
    const status = (error as { response?: { status?: number } })?.response?.status;
    saveError.value =
      status === 400
        ? "Check the task fields. Its project must match its parent and subtasks."
        : "Could not save the task. Check that it and its selected project and tags are still available, then try again.";
    return;
  }

  saved.value = true;
  emit("saved");
  emit("close");
  await refreshTaskLists();
}

onMounted(loadTask);
onScopeDispose(() => {
  disposed = true;
});
</script>

<template>
  <TaskModal
    v-if="task || loadError"
    :open="true"
    title="Edit task"
    :form-id="formId"
    :error-summary-id="errorSummaryId"
    :errors="errors"
    :is-submitting="isSubmitting"
    :save-disabled="!task || loading"
    @update:open="emit('close')"
  >
    <div v-if="loadError" class="p-6">
      <p role="alert" class="font-sans text-sm text-light-love dark:text-dark-love">
        {{ loadError }}
      </p>
      <Button v-if="taskId !== null" size="sm" class="mt-4" @click="loadTask">Try again</Button>
    </div>
    <TaskForm
      v-else-if="task"
      :initial-values="task"
      :form-id="formId"
      :error-summary-id="errorSummaryId"
      :submit-task="saveTask"
      @submit-start="saveError = ''"
      @errors="validationErrors = $event"
      @submitting="isSubmitting = $event"
      @dirty="dirty = $event"
    />
  </TaskModal>
</template>
