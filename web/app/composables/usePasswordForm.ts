import type { MaybeRefOrGetter } from "vue";
import { useForm } from "vee-validate";
import {
  profileErrorMessage,
  profileErrorStatus,
  validateProfilePassword,
  type ProfileFeedback,
} from "~/utils/profile";

export function usePasswordForm(disabled: MaybeRefOrGetter<boolean>) {
  type PasswordFormValues = typeof emptyValues;

  const emptyValues = { current_password: "", new_password: "", password_confirmation: "" };
  const { changePassword } = useProfile();
  const feedback = ref<ProfileFeedback>(null);
  const dirty = computed(() => meta.value.dirty);

  const { handleSubmit, meta, isSubmitting, resetForm, setFieldError, values } =
    useForm<PasswordFormValues>({
      initialValues: { ...emptyValues },
      validationSchema: {
        current_password: validateProfilePassword,
        new_password: validateProfilePassword,
        password_confirmation: (value: string | undefined) =>
          (!!value && value === values.new_password) || "The new passwords must match.",
      },
    });

  watch(
    values,
    () => {
      feedback.value = null;
    },
    { deep: true },
  );

  const submit = handleSubmit(async (formValues) => {
    feedback.value = null;
    try {
      await changePassword(formValues);
      resetForm({ values: { ...emptyValues } });
      await nextTick();

      feedback.value = {
        kind: "success",
        message: "Password changed. Other sessions have been signed out.",
      };
    } catch (error) {
      if (profileErrorStatus(error) === 403) {
        setFieldError("current_password", "The current password is incorrect.");
        return;
      }
      feedback.value = { kind: "error", message: profileErrorMessage(error) };
    }
  });

  function onSubmit(event: Event) {
    if (toValue(disabled) || isSubmitting.value) {
      event.preventDefault();
      return;
    }

    return submit(event);
  }

  function discard() {
    resetForm({ values: { ...emptyValues } });
    feedback.value = null;
  }

  return { onSubmit, discard, feedback, isSubmitting, dirty };
}
