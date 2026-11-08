<script setup lang="ts">
defineProps<{ titleId: string; title: string; description: string }>();
</script>

<template>
  <section
    class="login-card relative w-full rounded-2xl border border-light-highlight-med/70 bg-light-surface p-7 sm:p-10 dark:border-dark-highlight-med dark:bg-dark-surface"
    :aria-labelledby="titleId"
  >
    <div class="mb-8">
      <h1 :id="titleId" class="font-serif text-3xl font-semibold tracking-tight sm:text-4xl">
        {{ title }}
      </h1>
      <p class="mt-3 font-sans text-sm leading-6 text-light-subtle dark:text-dark-subtle">
        {{ description }}
      </p>
    </div>
    <slot />
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

.login-card :deep(.login-submit) {
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
