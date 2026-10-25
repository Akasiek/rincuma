<script setup lang="ts">
import { useForm } from "vee-validate";
import {
  profileFieldValidation,
  profileErrorMessage,
  profileErrorStatus,
  validateProfileEmail,
  type ProfileFeedback,
} from "~/utils/profile";

const props = defineProps<{ disabled: boolean }>();
const dirty = defineModel<boolean>("dirty", { default: false });
const saving = defineModel<boolean>("saving", { default: false });
const userStore = useUserStore();
const { updateProfile } = useProfile();
const feedback = ref<ProfileFeedback>(null);
const { handleSubmit, meta, isSubmitting, resetForm, setFieldError, values } = useForm({
  initialValues: { email: userStore.user?.email ?? "" },
  validationSchema: { email: validateProfileEmail },
});
watch(
  () => meta.value.dirty,
  (value) => {
    dirty.value = value;
  },
  { immediate: true },
);
watch(isSubmitting, (value) => {
  saving.value = value;
});
watch(
  values,
  () => {
    feedback.value = null;
  },
  { deep: true },
);

const submit = handleSubmit(async ({ email }) => {
  feedback.value = null;
  try {
    const user = await updateProfile(email.trim().toLowerCase());
    resetForm({ values: { email: user.email } });
    await nextTick();
    feedback.value = {
      kind: "success",
      message: "Profile saved. Other sessions have been signed out.",
    };
  } catch (error) {
    if (profileErrorStatus(error) === 409) {
      setFieldError("email", "This email address is already in use.");
      return;
    }
    feedback.value = { kind: "error", message: profileErrorMessage(error) };
  }
});

function onSubmit(event: Event) {
  if (props.disabled || isSubmitting.value) {
    event.preventDefault();
    return;
  }
  return submit(event);
}

function discard() {
  resetForm({ values: { email: userStore.user?.email ?? "" } });
  feedback.value = null;
}
</script>

<template>
  <form novalidate :aria-busy="isSubmitting" @submit="onSubmit">
    <ProfileFormSection
      title="User details"
      description="Update the email address you use to log in. Other sessions will be signed out after a change."
      :saving="isSubmitting"
      :dirty="meta.dirty"
      :disabled="disabled || isSubmitting"
      :feedback="feedback"
      @discard="discard"
    >
      <FormField
        v-bind="profileFieldValidation"
        id="profile-email"
        name="email"
        label="Email"
        type="email"
        autocomplete="email"
        autocapitalize="none"
        spellcheck="false"
      />
    </ProfileFormSection>
  </form>
</template>
