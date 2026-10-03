<script setup lang="ts">
import TaskForm from "./Form.vue";
import TaskCreatorModal from "./creator/Modal.vue";

const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ saved: [] }>();
const { createTask } = useTask();
const formId = useId();
const errorSummaryId = `${formId}-errors`;
const validationErrors = ref<{ fieldId?: string; message: string }[]>([]);
const saveError = ref("");
const isSubmitting = ref(false);

const errors = computed(() => {
  const messages = [...validationErrors.value];
  if (saveError.value) messages.push({ message: saveError.value });
  return messages;
});

watch(open, (isOpen) => {
  if (isOpen) {
    validationErrors.value = [];
    saveError.value = "";
  }
});

async function saveTask(values: SaveTaskRequest) {
  try {
    await createTask(values);
    open.value = false;
    emit("saved");
  } catch {
    saveError.value = "Save: Could not save the task. Please try again.";
  }
}
</script>

<template>
  <TaskCreatorModal
    v-model:open="open"
    :form-id="formId"
    :error-summary-id="errorSummaryId"
    :errors="errors"
    :is-submitting="isSubmitting"
  >
    <template v-if="$slots.trigger" #trigger>
      <slot name="trigger" />
    </template>

    <TaskForm
      :form-id="formId"
      :error-summary-id="errorSummaryId"
      :submit-task="saveTask"
      @submit-start="saveError = ''"
      @errors="validationErrors = $event"
      @submitting="isSubmitting = $event"
    />
  </TaskCreatorModal>
</template>
