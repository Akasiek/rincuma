<script setup lang="ts">
import type { SubmissionHandler } from "vee-validate";

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
    loginError.value = "Nie udało się zalogować. Sprawdź dane i spróbuj ponownie.";
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
  <div class="min-h-screen flex items-center justify-center">
    <Form
      :validation-schema="schema"
      @submit="onSubmit"
      class="p-4 flex flex-col gap-6 items-center"
    >
      <div>
        <Field name="email" as="input" class="border-2 border-zinc-900" />
        <ErrorMessage name="email" />
      </div>

      <div>
        <Field name="password" as="input" type="password" class="border-2 border-zinc-900" />
        <ErrorMessage name="password" />
      </div>

      <div class="text-red-500 font-bold">
        {{ loginError }}
      </div>

      <div>
        <button class="border-2 py-1 px-2.5 border-zinc-900">Log in</button>
      </div>
    </Form>
  </div>
</template>
