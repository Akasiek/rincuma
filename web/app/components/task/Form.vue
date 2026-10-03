<script setup lang="ts">
const { formId, errorSummaryId, submitTask } = defineProps<{
  formId: string;
  errorSummaryId: string;
  submitTask: (values: SaveTaskRequest) => Promise<void>;
}>();

const emit = defineEmits<{
  submitStart: [];
  errors: [errors: { fieldId?: string; message: string }[]];
  submitting: [submitting: boolean];
}>();

const fieldLabels: Record<keyof SaveTaskRequest, string> = {
  name: "Name",
  description: "Description",
  due_at: "Date and time",
  priority: "Priority",
  project_id: "Project",
  parent_id: "Parent task",
  tag_ids: "Tags",
};

const { handleSubmit, defineField, errors, errorBag, isSubmitting } = useForm<SaveTaskRequest>({
  validationSchema: {
    name: (value: string) => {
      const name = value?.trim() ?? "";
      if (!name) return "Enter a task name.";
      if (new TextEncoder().encode(name).length > 255) {
        return "Task name must not exceed 255 bytes.";
      }
      return true;
    },
  },
  initialValues: {
    name: "",
    description: null,
    due_at: null,
    priority: "none",
    project_id: null,
    parent_id: null,
    tag_ids: [],
  },
});

const [dueAt] = defineField("due_at");

const errorMessages = computed<{ fieldId?: string; message: string }[]>(() => {
  const messages: { fieldId?: string; message: string }[] = Object.entries(errorBag.value).flatMap(
    ([field, fieldErrors]) =>
      (fieldErrors ?? []).map((message) => ({
        fieldId: `${formId}-${field}`,
        message: `${fieldLabels[field as keyof SaveTaskRequest] ?? field}: ${message}`,
      })),
  );
  return messages;
});

watch(errorMessages, (messages) => emit("errors", messages), { immediate: true });
watch(isSubmitting, (submitting) => emit("submitting", submitting), { immediate: true });

const submit = handleSubmit((values) => submitTask(values));

function onSubmit(event: Event) {
  if (isSubmitting.value) {
    event.preventDefault();
    return;
  }
  emit("submitStart");
  return submit(event);
}
</script>

<template>
  <form
    :id="formId"
    @submit="onSubmit"
    class="flex flex-col rounded-md bg-light-base p-4 pb-10 dark:bg-dark-base"
  >
    <Field
      :id="`${formId}-name`"
      name="name"
      label="Name"
      aria-label="Name"
      :aria-invalid="!!errors.name"
      :aria-describedby="errors.name ? errorSummaryId : undefined"
      :class="{ 'ring-2 ring-light-love dark:ring-dark-love': errors.name }"
      as="input"
      class="w-full rounded-md p-4 text-2xl font-black focus:outline-0"
      placeholder="Task name..."
    />

    <div
      class="mt-1 mb-3 flex w-full items-center justify-between border-y-2 border-light-highlight-med py-2 dark:border-dark-highlight-med"
    >
      <TimeDatePicker v-model="dueAt" />
    </div>

    <Field
      :id="`${formId}-description`"
      name="description"
      label="Description"
      aria-label="Description"
      :aria-invalid="!!errors.description"
      :aria-describedby="errors.description ? errorSummaryId : undefined"
      :class="{ 'ring-2 ring-light-love dark:ring-dark-love': errors.description }"
      as="textarea"
      class="max-h-80 min-h-26 w-full rounded-md px-4 py-2 text-lg focus:outline-0"
      placeholder="Something more?"
    />
  </form>
</template>
