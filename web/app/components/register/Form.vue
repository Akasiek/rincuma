<script setup lang="ts">
import { Form as VeeForm, type SubmissionHandler } from "vee-validate";
import { profileErrorStatus, validateProfilePassword } from "~/utils/profile";

type RegisterValues = { email: string; password: string; password_confirmation: string };

const { register } = useAuth();

const schema = {
  email: validateEmail,
  password: validateProfilePassword,
  password_confirmation: validateConfirmation,
};

const registerError = ref("");
const isSubmitting = ref(false);

function validateEmail(value: string | undefined) {
  return /^\S+@\S+$/.test(value?.trim() ?? "") || "Enter a valid email address.";
}

function validateConfirmation(value: string | undefined, context: { form: RegisterValues }) {
  return (!!value && value === context.form.password) || "The passwords must match.";
}

async function handleRegisterError(error: unknown, setEmailError: (message: string) => void) {
  const status = profileErrorStatus(error);

  if (status === 409) {
    setEmailError("An account with this email already exists.");
    return;
  }
  if (status === 403) {
    await navigateTo("/login", { replace: true });
    return;
  }

  registerError.value = "Could not create your account. Please try again.";
}

const onSubmit: SubmissionHandler = async (values, { setFieldError }) => {
  registerError.value = "";
  isSubmitting.value = true;

  try {
    await register({ email: values.email, password: values.password });
    await navigateTo("/");
  } catch (error) {
    await handleRegisterError(error, (message) => setFieldError("email", message));
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<template>
  <LoginCard
    title-id="register-title"
    title="Create your account."
    description="Start organizing your tasks in one place."
  >
    <VeeForm
      :validation-schema="schema"
      novalidate
      :aria-busy="isSubmitting"
      aria-labelledby="register-title"
      @submit="onSubmit"
      class="mb-2"
    >
      <fieldset :disabled="isSubmitting" class="space-y-5">
        <legend class="sr-only">Account details</legend>
        <FormField
          id="register-email"
          name="email"
          label="Email"
          type="email"
          autocomplete="username"
          autocapitalize="none"
          spellcheck="false"
          placeholder="you@example.com"
          class="login-input"
        />
        <FormField
          id="register-password"
          name="password"
          label="Password"
          type="password"
          autocomplete="new-password"
          placeholder="Choose a password"
          class="login-input"
        />
        <FormField
          id="register-password-confirmation"
          name="password_confirmation"
          label="Confirm password"
          type="password"
          autocomplete="new-password"
          placeholder="Repeat your password"
          class="login-input"
        />
        <LoginErrorAlert v-if="registerError" :message="registerError" />
        <Button
          type="submit"
          variant="primary"
          full-width
          :loading="isSubmitting"
          class="login-submit mt-2"
        >
          {{ isSubmitting ? "Creating account…" : "Create account" }}
          <Icon v-if="!isSubmitting" name="carbon:arrow-right" class="size-4" aria-hidden="true" />
        </Button>
      </fieldset>
    </VeeForm>
    <p class="mt-6 text-center font-sans text-sm text-light-subtle dark:text-dark-subtle">
      Already have an account?
      <NuxtLink
        to="/login"
        class="font-medium text-light-pine underline-offset-4 hover:underline dark:text-dark-foam"
      >
        Log in
      </NuxtLink>
    </p>
  </LoginCard>
</template>
