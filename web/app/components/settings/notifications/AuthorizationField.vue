<script setup lang="ts">
const authorization = defineModel<string>({ required: true });
const remove = defineModel<boolean>("remove", { required: true });

defineProps<{
  hasAuthorization: boolean;
  disabled: boolean;
  error?: string;
}>();

function toggleRemoval() {
  remove.value = !remove.value;
  authorization.value = "";
}
</script>

<template>
  <FormGroup
    id="notification-authorization"
    label="Authorization"
    optional
    :error="error"
    :description="
      remove
        ? 'Saved authorization will be removed when you save changes.'
        : 'Enter a Bearer token or Basic authorization header. Saved credentials are never displayed.'
    "
  >
    <template #actions>
      <button
        v-if="hasAuthorization"
        type="button"
        :disabled="disabled"
        class="cursor-pointer font-sans text-xs text-light-love underline-offset-4 hover:underline disabled:cursor-not-allowed disabled:opacity-50 dark:text-dark-love"
        @click="toggleRemoval"
      >
        {{ remove ? "Keep saved authorization" : "Remove saved authorization" }}
      </button>
    </template>
    <template #default="{ describedby, invalid }">
      <FormInput
        id="notification-authorization"
        v-model="authorization"
        name="authorization"
        type="password"
        autocomplete="new-password"
        spellcheck="false"
        autocapitalize="none"
        :disabled="disabled || remove"
        :placeholder="
          hasAuthorization ? 'Leave blank to keep saved authorization' : 'Bearer tk_… or Basic …'
        "
        :invalid="invalid"
        :aria-describedby="describedby"
      />
    </template>
  </FormGroup>
</template>
