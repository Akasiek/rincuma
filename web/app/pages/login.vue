<script setup lang="ts">
import type { SubmissionHandler } from "vee-validate";

useHead({ title: "Log in" });

const { object, string } = useYup();
const { login } = useAuth();
const { isLoggedIn } = useUserStore();

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

onMounted(async () => {
  if (isLoggedIn) {
    await navigateTo("/");
  }
});
</script>

<template>
  <main class="flex min-h-screen items-center justify-center px-4">
    <Form
      :validation-schema="schema"
      novalidate
      :aria-busy="isSubmitting"
      class="w-full max-w-sm rounded-lg border border-light-highlight-med bg-light-surface p-6 dark:border-dark-highlight-med dark:bg-dark-surface"
      @submit="onSubmit"
    >
      <h1 class="mb-6 text-center text-4xl font-black">Rincuma</h1>
      <fieldset :disabled="isSubmitting" class="space-y-6">
        <legend class="sr-only">Login credentials</legend>
        <FormField
          id="login-email"
          name="email"
          label="Email"
          type="email"
          autocomplete="username"
          autocapitalize="none"
          spellcheck="false"
        />
        <FormField
          id="login-password"
          name="password"
          label="Password"
          type="password"
          autocomplete="current-password"
        />
        <p
          v-if="loginError"
          role="alert"
          class="font-sans text-sm text-light-love dark:text-dark-love"
        >
          {{ loginError }}
        </p>
        <Button type="submit" variant="primary" full-width :loading="isSubmitting">{{
          isSubmitting ? "Logging in…" : "Log in"
        }}</Button>
      </fieldset>
    </Form>
  </main>
</template>
