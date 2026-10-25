<script setup lang="ts">
import TaskForm from "./Form.vue";
import TaskModal from "./Modal.vue";

const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ saved: [] }>();
const { createTask, refreshTaskLists } = useTask();
const route = useRoute();
const formId = useId();
const errorSummaryId = `${formId}-errors`;
const validationErrors = ref<{ fieldId?: string; message: string }[]>([]);
const saveError = ref("");
const isSubmitting = ref(false);
const dirty = ref(false);
const saved = ref(false);
const formKey = ref(0);
const confirmClose = useUnsavedChanges({
  dirty: computed(() => open.value && dirty.value && !saved.value),
  saving: computed(() => open.value && isSubmitting.value && !saved.value),
});

const errors = computed(() => {
  const messages = [...validationErrors.value];
  if (saveError.value) {
    messages.push({ message: saveError.value });
  }
  return messages;
});

const modalOpen = computed({
  get: () => open.value,
  set: (value: boolean) => {
    if (value) {
      open.value = true;
      return;
    }
    void close();
  },
});

async function close() {
  if (await confirmClose()) {
    open.value = false;
  }
}

watch(open, (isOpen) => {
  if (isOpen) {
    validationErrors.value = [];
    saveError.value = "";
    dirty.value = false;
    saved.value = false;
    formKey.value += 1;
  }
});
watch(
  () => route.fullPath,
  () => {
    open.value = false;
  },
);

async function saveTask(values: SaveTaskRequest) {
  try {
    await createTask(values);
  } catch {
    saveError.value = "Save: Could not save the task. Please try again.";
    return;
  }
  saved.value = true;
  open.value = false;
  emit("saved");
  await refreshTaskLists();
}
</script>

<template>
  <TaskModal
    v-model:open="modalOpen"
    title="New task"
    :form-id="formId"
    :error-summary-id="errorSummaryId"
    :errors="errors"
    :is-submitting="isSubmitting"
  >
    <template v-if="$slots.trigger" #trigger><slot name="trigger" /></template>
    <TaskForm
      :key="formKey"
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
