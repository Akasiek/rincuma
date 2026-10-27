<script setup lang="ts">
import { Form as VeeForm, type SubmissionHandler } from "vee-validate";

const { object, string } = useYup();
const { login } = useAuth();

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
  <section
    class="login-card relative w-full rounded-2xl border border-light-highlight-med/70 bg-light-surface p-7 sm:p-10 dark:border-dark-highlight-med dark:bg-dark-surface"
    aria-labelledby="login-title"
  >
    <div class="mb-8">
      <h1 id="login-title" class="font-serif text-3xl font-semibold tracking-tight sm:text-4xl">
        Welcome back.
      </h1>
      <p class="mt-3 font-sans text-sm leading-6 text-light-subtle dark:text-dark-subtle">
        Log in to pick up where you left off.
      </p>
    </div>
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
        <div
          v-if="loginError"
          role="alert"
          class="flex items-start gap-2.5 rounded-lg border border-light-love/20 bg-light-love/5 px-3 py-3 font-sans text-sm leading-6 text-light-love dark:border-dark-love/20 dark:bg-dark-love/5 dark:text-dark-love"
        >
          <Icon name="carbon:warning-alt" class="mt-1 size-4 shrink-0" aria-hidden="true" />
          <p>{{ loginError }}</p>
        </div>
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
  </section>
</template>

<style scoped>
.login-card {
  box-shadow: 0 24px 70px -32px color-mix(in srgb, var(--color-light-text) 25%, transparent);
}

.login-card :deep(.login-input) {
  min-height: 3rem;
  border-width: 1px;
  border-radius: 0.5rem;
  background-color: var(--color-light-base);
  transition:
    border-color 150ms,
    box-shadow 150ms;
}

.login-card :deep(.login-input:focus-visible) {
  outline-color: var(--color-light-pine);
  outline-offset: 3px;
}

.login-card :deep(.login-input::placeholder) {
  color: var(--color-light-muted);
}

.login-submit {
  min-height: 3rem;
  border-radius: 0.5rem;
}

@media (prefers-color-scheme: dark) {
  .login-card {
    box-shadow: 0 24px 70px -32px rgb(0 0 0 / 45%);
  }

  .login-card :deep(.login-input) {
    background-color: var(--color-dark-base);
  }

  .login-card :deep(.login-input:focus-visible) {
    outline-color: var(--color-dark-foam);
  }

  .login-card :deep(.login-input::placeholder) {
    color: var(--color-dark-muted);
  }
}
</style>
