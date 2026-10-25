<script setup lang="ts">
import type { SaveTaskRequest } from "#shared/types/task";
import { taskFormValues } from "~/utils/taskForm";

const { formId, errorSummaryId, submitTask, initialValues } = defineProps<{
  initialValues?: SaveTaskRequest;
  formId: string;
  errorSummaryId: string;
  submitTask: (values: SaveTaskRequest) => Promise<void>;
}>();

const emit = defineEmits<{
  submitStart: [];
  dirty: [dirty: boolean];
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

const fieldValidation = {
  validateOnBlur: false,
  validateOnChange: false,
  validateOnInput: false,
  validateOnModelUpdate: false,
};

const { handleSubmit, defineField, errors, errorBag, isSubmitting, meta } =
  useForm<SaveTaskRequest>({
    validateOnMount: false,
    validationSchema: {
      name: (value: string) => {
        const name = value?.trim() ?? "";
        if (!name) {
          return "Enter a task name.";
        }
        if (new TextEncoder().encode(name).length > 255) {
          return "Task name must not exceed 255 bytes.";
        }
        return true;
      },
    },
    initialValues: taskFormValues(initialValues),
  });

const [dueAt] = defineField("due_at", fieldValidation);
const [projectId] = defineField("project_id", fieldValidation);
const [tagIds] = defineField("tag_ids", fieldValidation);
const [priority] = defineField("priority", fieldValidation);

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

watch(
  () => meta.value.dirty,
  (dirty) => emit("dirty", dirty),
  { immediate: true },
);

const submit = handleSubmit((values) => submitTask(taskFormValues(values)));

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
    novalidate
    :aria-busy="isSubmitting"
    @submit="onSubmit"
    class="flex flex-col rounded-md bg-light-base p-4 pb-10 dark:bg-dark-base"
  >
    <fieldset :disabled="isSubmitting" class="contents">
      <Field v-slot="{ componentField }" v-bind="fieldValidation" name="name" label="Name">
        <FormInput
          v-bind="componentField"
          :id="`${formId}-name`"
          variant="task"
          aria-label="Name"
          :invalid="!!errors.name"
          :aria-describedby="errors.name ? errorSummaryId : undefined"
          placeholder="Task name..."
        />
      </Field>

      <div
        class="mt-1 mb-3 flex w-full min-w-0 items-center gap-1 border-y-2 border-light-highlight-med py-2 dark:border-dark-highlight-med"
      >
        <div class="min-w-0 flex-1">
          <TimeDatePicker v-model="dueAt" compact />
        </div>
        <ProjectSelect
          v-model="projectId"
          class="min-w-0"
          :id="`${formId}-project_id`"
          :retained-project-id="initialValues?.project_id"
          compact
          :disabled="isSubmitting"
          :invalid="!!errors.project_id"
          :describedby="errors.project_id ? errorSummaryId : undefined"
        />
        <TagSelect
          v-model="tagIds"
          class="max-w-28 min-w-0"
          :id="`${formId}-tag_ids`"
          compact
          :disabled="isSubmitting"
          :invalid="!!errors.tag_ids"
          :describedby="errors.tag_ids ? errorSummaryId : undefined"
        />
        <TaskPrioritySelect
          v-model="priority"
          class="max-w-28 min-w-0"
          :id="`${formId}-priority`"
          compact
          :disabled="isSubmitting"
          :invalid="!!errors.priority"
          :describedby="errors.priority ? errorSummaryId : undefined"
        />
      </div>

      <Field
        v-slot="{ componentField }"
        v-bind="fieldValidation"
        name="description"
        label="Description"
      >
        <FormTextarea
          v-bind="componentField"
          :id="`${formId}-description`"
          variant="task"
          aria-label="Description"
          :invalid="!!errors.description"
          :aria-describedby="errors.description ? errorSummaryId : undefined"
          class="max-h-80"
          placeholder="Something more?"
        />
      </Field>
    </fieldset>
  </form>
</template>
