<script setup lang="ts">
import { profileFieldValidation } from "~/utils/profile";

const props = defineProps<{ disabled: boolean }>();
const dirty = defineModel<boolean>("dirty", { default: false });
const saving = defineModel<boolean>("saving", { default: false });
const {
  onSubmit,
  discard,
  feedback,
  isSubmitting,
  dirty: formDirty,
} = usePasswordForm(() => props.disabled);

watch(
  formDirty,
  (value) => {
    dirty.value = value;
  },
  { immediate: true },
);

watch(isSubmitting, (value) => {
  saving.value = value;
});
</script>

<template>
  <form novalidate :aria-busy="isSubmitting" @submit="onSubmit">
    <ProfileFormSection
      title="Password"
      description="Choose a new password. Other sessions will be signed out after this change."
      :saving="isSubmitting"
      :dirty="formDirty"
      :disabled="disabled || isSubmitting"
      :feedback="feedback"
      @discard="discard"
    >
      <FormField
        v-bind="profileFieldValidation"
        id="profile-current-password"
        name="current_password"
        label="Current password"
        type="password"
        autocomplete="current-password"
      />
      <FormField
        v-bind="profileFieldValidation"
        id="profile-new-password"
        name="new_password"
        label="New password"
        type="password"
        autocomplete="new-password"
      />
      <FormField
        v-bind="profileFieldValidation"
        id="profile-password-confirmation"
        name="password_confirmation"
        label="Confirm new password"
        type="password"
        autocomplete="new-password"
      />
    </ProfileFormSection>
  </form>
</template>
