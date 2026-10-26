<script setup lang="ts">
import type { SaveTaskRequest } from "#shared/types/task";
import TaskForm from "./Form.vue";
import TaskModal from "./Modal.vue";

const props = defineProps<{ taskId: number | null }>();
const emit = defineEmits<{ close: []; saved: [] }>();
const { getTask, updateTask, refreshTaskLists } = useTask();
const userStore = useUserStore();
const taskRequest = useAsyncData(
  `task-editor:${userStore.user?.id ?? "guest"}:${props.taskId}`,
  () => {
    if (props.taskId === null) {
      throw createError({ status: 400, message: "This task link is invalid." });
    }

    return getTask(props.taskId);
  },
);
const { data: task, status, error: taskError, refresh } = taskRequest;
const loading = computed(() => status.value === "pending");
const loadError = computed(() => {
  if (props.taskId === null) {
    return "This task link is invalid.";
  }
  if (!taskError.value) {
    return "";
  }

  return taskError.value.status === 404
    ? "This task is no longer available."
    : "Could not load this task. Please try again.";
});
const recurrenceReadonly = computed(
  () => !!task.value?.completed_at || (!!task.value?.series_id && !task.value?.recurrence),
);
const saveError = ref("");
const validationErrors = ref<{ fieldId?: string; message: string }[]>([]);
const isSubmitting = ref(false);
const dirty = ref(false);
const saved = ref(false);
const formId = useId();
const errorSummaryId = `${formId}-errors`;
onScopeDispose(taskRequest.clear);

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

async function saveTask(values: SaveTaskRequest) {
  if (!task.value) {
    return;
  }

  try {
    await updateTask(task.value.id, {
      ...values,
      parent_id: task.value.parent_id,
      recurrence: recurrenceReadonly.value ? undefined : values.recurrence,
    });
  } catch (error) {
    const status = (error as { response?: { status?: number } })?.response?.status;
    if (status === 409) {
      saveError.value =
        "The task or its series changed. Reopen the editor and try again. Only an active task without subtasks can repeat.";
      return;
    }
    saveError.value =
      status === 400
        ? "Check the task fields and repeat settings. Its project must match its parent and subtasks."
        : "Could not save the task. Check that it and its selected project and tags are still available, then try again.";
    return;
  }

  saved.value = true;
  emit("saved");
  emit("close");
  await refreshTaskLists();
}

await taskRequest;
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
      <Button v-if="taskId !== null" size="sm" class="mt-4" :loading="loading" @click="refresh()">
        Try again
      </Button>
    </div>
    <TaskForm
      v-else-if="task"
      :initial-values="task"
      :recurrence-readonly="recurrenceReadonly"
      :occurrence-count="task.recurrence?.occurrence_count"
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
