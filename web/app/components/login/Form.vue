<script setup lang="ts">
import { Form as VeeForm, type SubmissionHandler } from "vee-validate";

const { object, string } = useYup();
const { login } = useAuth();
const instance = useInstanceStore();

const schema = toTypedSchema(
  object({
    email: string().required().label("E-mail"),
    password: string().required().label("Password"),
  }),
);

const loginError = ref("");
const isSubmitting = ref(false);

const onSubmit: SubmissionHandler = async (values) => {
  loginError.value = "";
  isSubmitting.value = true;

  try {
    await login({
      email: values.email,
      password: values.password,
    });

    await navigateTo("/");
  } catch {
    loginError.value = "Could not log in. Check your credentials and try again.";
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<template>
  <LoginCard
    title-id="login-title"
    title="Welcome back."
    description="Log in to pick up where you left off."
  >
    <VeeForm
      :validation-schema="schema"
      novalidate
      :aria-busy="isSubmitting"
      aria-labelledby="login-title"
      @submit="onSubmit"
      class="mb-2"
    >
      <fieldset :disabled="isSubmitting" class="space-y-5">
        <legend class="sr-only">Login credentials</legend>
        <FormField
          id="login-email"
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
          id="login-password"
          name="password"
          label="Password"
          type="password"
          autocomplete="current-password"
          placeholder="Enter your password"
          class="login-input"
        />
        <LoginErrorAlert v-if="loginError" :message="loginError" />
        <Button
          type="submit"
          variant="primary"
          full-width
          :loading="isSubmitting"
          class="login-submit mt-2"
        >
          {{ isSubmitting ? "Logging in…" : "Log in" }}
          <Icon v-if="!isSubmitting" name="carbon:arrow-right" class="size-4" aria-hidden="true" />
        </Button>
      </fieldset>
    </VeeForm>
    <p
      v-if="instance.registrationEnabled"
      class="mt-6 text-center font-sans text-sm text-light-subtle dark:text-dark-subtle"
    >
      New here?
      <NuxtLink
        to="/register"
        class="font-medium text-light-pine underline-offset-4 hover:underline dark:text-dark-foam"
      >
        Create account
      </NuxtLink>
    </p>
  </LoginCard>
</template>
