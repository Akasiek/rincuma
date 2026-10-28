<script setup lang="ts">
import { Field, useForm } from "vee-validate";
import { useToastManager } from "reka-ui";
import {
  emptyDictionaryForm,
  validateDictionaryColor,
  validateDictionaryName,
  type DictionaryFormValues,
} from "~/utils/dictionaryCreation";

const props = defineProps<{
  kind: "project" | "tag";
  save: (values: DictionaryFormValues) => Promise<void>;
  initialValues?: DictionaryFormValues;
}>();
const open = defineModel<boolean>("open", { default: false });

const userStore = useUserStore();
const route = useRoute();
const formId = useId();
const toast = useToastManager();
const { handleSubmit, isSubmitting, meta, resetForm } = useForm<DictionaryFormValues>({
  initialValues: emptyDictionaryForm(),
  validationSchema: { name: validateDictionaryName, color: validateDictionaryColor },
});

const saved = ref(false);
const error = ref("");

const confirmClose = useUnsavedChanges({
  dirty: computed(() => open.value && meta.value.dirty && !saved.value),
  saving: computed(() => open.value && isSubmitting.value && !saved.value),
});

const editing = computed(() => !!props.initialValues);
const title = computed(() => `${editing.value ? "Edit" : "New"} ${props.kind}`);
const modalOpen = computed({
  get: () => open.value,
  set: (value: boolean) => {
    if (value) {
      open.value = true;
    } else {
      void close();
    }
  },
});

watch(open, (value) => {
  if (value) {
    resetForm({ values: props.initialValues ?? emptyDictionaryForm() });
    saved.value = false;
    error.value = "";
  }
});

watch(
  () => route.fullPath,
  () => {
    open.value = false;
  },
);

async function close() {
  if (!isSubmitting.value && (await confirmClose())) {
    open.value = false;
  }
}

async function focusName(event: Event) {
  event.preventDefault();
  await nextTick();
  document.getElementById(`${formId}-name`)?.focus();
}

const submit = handleSubmit(async (values) => {
  error.value = "";
  const ownerId = userStore.user?.id;

  try {
    await props.save({ ...values, name: values.name.trim() });
  } catch {
    error.value = `Could not save the ${props.kind}. Please try again.`;
    return;
  }

  saved.value = true;
  open.value = false;
  if (userStore.user?.id === ownerId) {
    toast.add({
      title: `${props.kind === "project" ? "Project" : "Tag"} ${editing.value ? "updated" : "created"}`,
      description: values.name.trim(),
      status: "success",
    });
  }
});

function onSubmit(event: Event) {
  if (isSubmitting.value) {
    event.preventDefault();
    return;
  }

  return submit(event);
}
</script>

<template>
  <AppModal
    v-model:open="modalOpen"
    :title="title"
    size="sm"
    :aria-busy="isSubmitting"
    @open-auto-focus="focusName"
  >
    <template v-if="$slots.trigger" #trigger><slot name="trigger" /></template>

    <form class="p-6" novalidate @submit="onSubmit">
      <h2 class="text-xl font-bold">{{ title }}</h2>

      <fieldset :disabled="isSubmitting" class="mt-5 space-y-5">
        <FormField :id="`${formId}-name`" name="name" label="Name" autocomplete="off" required />
        <FormField
          v-if="kind === 'project'"
          :id="`${formId}-description`"
          name="description"
          label="Description"
          as="textarea"
          optional
          class="min-h-20"
        />
        <Field v-slot="{ componentField, errorMessage }" name="color" label="Color">
          <FormGroup :id="`${formId}-color`" label="Color" optional :error="errorMessage">
            <template #default="{ describedby, invalid }">
              <FormColorInput
                v-bind="componentField"
                :id="`${formId}-color`"
                :describedby="describedby"
                :invalid="invalid"
                :disabled="isSubmitting"
              />
            </template>
          </FormGroup>
        </Field>
      </fieldset>

      <p
        v-if="error"
        role="alert"
        class="mt-4 font-sans text-sm text-light-love dark:text-dark-love"
      >
        {{ error }}
      </p>

      <div class="mt-6 flex justify-end gap-3">
        <Button :disabled="isSubmitting" @click="close">Cancel</Button>
        <Button type="submit" variant="primary" :loading="isSubmitting">
          {{ editing ? "Save changes" : `Create ${kind}` }}
        </Button>
      </div>
    </form>
  </AppModal>
</template>
