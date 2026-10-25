<script setup lang="ts">
import { useForm } from "vee-validate";
import { SwitchRoot, SwitchThumb } from "reka-ui";
import type { NotificationSettings } from "#shared/types/notification";
import type { SelectOption } from "#shared/types/select";
import {
  notificationFormValues,
  notificationSettingsRequest,
  validateNotificationAuthorization,
  validateNtfyTopicUrl,
  type NotificationFormValues,
} from "~/utils/notificationSettings";

const props = defineProps<{ settings: NotificationSettings | null }>();
const emit = defineEmits<{ saved: [settings: NotificationSettings] }>();
const { saveSettings } = useNotificationSettings();
const removeAuthorization = ref(false);
const feedback = ref<{ kind: "success" | "error"; message: string } | null>(null);
const fieldValidation = {
  validateOnBlur: false,
  validateOnChange: false,
  validateOnInput: false,
  validateOnModelUpdate: false,
};
const { handleSubmit, defineField, errors, isSubmitting, meta, resetForm, values } =
  useForm<NotificationFormValues>({
    initialValues: notificationFormValues(props.settings),
    validateOnMount: false,
    validationSchema: {
      provider: (value: string) => value === "ntfy" || "Choose a supported provider.",
      url: validateNtfyTopicUrl,
      authorization: validateNotificationAuthorization,
    },
  });

const [enabled] = defineField("enabled", fieldValidation);
const [authorization] = defineField("authorization", fieldValidation);
const providerOptions = computed(() => {
  const options: SelectOption[] = [{ value: "ntfy", label: "ntfy" }];
  if (props.settings && props.settings.provider !== "ntfy") {
    options.unshift({
      value: props.settings.provider,
      label: `${props.settings.provider} (not supported)`,
      disabled: true,
    });
  }
  return options;
});
const hasChanges = computed(() => meta.value.dirty || removeAuthorization.value);

watch(
  [values, removeAuthorization],
  () => {
    feedback.value = null;
  },
  { deep: true },
);

const submit = handleSubmit(async (formValues) => {
  feedback.value = null;
  try {
    const settings = await saveSettings(
      notificationSettingsRequest(formValues, removeAuthorization.value),
    );
    emit("saved", settings);
    resetForm({ values: notificationFormValues(settings) });
    removeAuthorization.value = false;
    await nextTick();
    feedback.value = { kind: "success", message: "Notification settings saved." };
  } catch {
    feedback.value = {
      kind: "error",
      message: "Could not save your settings. Your changes are still here. Please try again.",
    };
  }
});

function onSubmit(event: Event) {
  if (isSubmitting.value) {
    event.preventDefault();
    return;
  }
  return submit(event);
}

function discardChanges() {
  resetForm({ values: notificationFormValues(props.settings) });
  removeAuthorization.value = false;
  feedback.value = null;
}
</script>

<template>
  <form novalidate :aria-busy="isSubmitting" @submit="onSubmit" class="max-w-2xl font-sans">
    <fieldset
      :disabled="isSubmitting"
      class="min-w-0 rounded-lg border border-light-highlight-med bg-light-surface dark:border-dark-highlight-med dark:bg-dark-surface"
    >
      <legend class="sr-only">Notification delivery</legend>
      <div
        class="flex items-center justify-between gap-6 border-b border-light-highlight-med p-5 sm:p-6 dark:border-dark-highlight-med"
      >
        <div>
          <label for="notification-enabled" class="cursor-pointer font-serif font-bold">
            Task reminders
          </label>
          <p
            id="notification-enabled-help"
            class="mt-1 text-xs leading-relaxed text-light-subtle dark:text-dark-subtle"
          >
            {{
              enabled
                ? "Send a separate notification for each task when it is due."
                : "Reminders are paused. You can still configure your connection."
            }}
          </p>
        </div>
        <SwitchRoot
          id="notification-enabled"
          v-model="enabled"
          :disabled="isSubmitting"
          aria-describedby="notification-enabled-help"
          class="flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full bg-light-highlight-high p-0.5 transition-colors disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-light-pine dark:bg-dark-highlight-high dark:data-[state=checked]:bg-dark-pine"
        >
          <SwitchThumb
            class="block size-5 rounded-full bg-light-surface shadow-sm transition-transform data-[state=checked]:translate-x-5"
          />
        </SwitchRoot>
      </div>

      <div class="space-y-6 p-5 sm:p-6">
        <FormField
          v-bind="fieldValidation"
          id="notification-provider"
          name="provider"
          label="Provider"
          as="select"
          :options="providerOptions"
        />

        <FormField
          v-bind="fieldValidation"
          id="notification-url"
          name="url"
          label="Topic URL"
          type="url"
          autocomplete="url"
          placeholder="https://ntfy.example.com/my-tasks"
          spellcheck="false"
          autocapitalize="none"
          description="The full address of your ntfy topic, including its name."
        />

        <SettingsNotificationsAuthorizationField
          v-model="authorization"
          v-model:remove="removeAuthorization"
          :has-authorization="settings?.has_authorization ?? false"
          :disabled="isSubmitting"
          :error="errors.authorization"
        />
      </div>
    </fieldset>

    <p
      v-if="feedback"
      :role="feedback.kind === 'error' ? 'alert' : 'status'"
      :class="
        feedback.kind === 'error'
          ? 'text-light-love dark:text-dark-love'
          : 'text-light-pine dark:text-dark-foam'
      "
      class="mt-4 text-sm leading-relaxed"
    >
      {{ feedback.message }}
    </p>

    <div class="mt-6 flex flex-wrap items-center gap-3">
      <Button
        type="submit"
        variant="primary"
        size="sm"
        :loading="isSubmitting"
        :disabled="isSubmitting || (!hasChanges && !!settings)"
      >
        <Icon
          v-if="isSubmitting"
          name="carbon:circle-dash"
          class="size-4 animate-spin motion-reduce:animate-none"
          aria-hidden="true"
        />
        {{ isSubmitting ? "Saving…" : "Save changes" }}
      </Button>
      <Button
        type="button"
        size="sm"
        :disabled="isSubmitting || !hasChanges"
        @click="discardChanges"
      >
        Discard changes
      </Button>
    </div>
  </form>
</template>
