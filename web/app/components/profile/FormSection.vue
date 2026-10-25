<script setup lang="ts">
import type { ProfileFeedback } from "~/utils/profile";

defineProps<{
  title: string;
  description: string;
  saving: boolean;
  dirty: boolean;
  disabled: boolean;
  feedback: ProfileFeedback;
}>();

const emit = defineEmits<{ discard: [] }>();
</script>

<template>
  <fieldset
    :disabled="disabled"
    class="min-w-0 rounded-lg border border-light-highlight-med bg-light-surface p-5 sm:p-6 dark:border-dark-highlight-med dark:bg-dark-surface"
  >
    <legend class="sr-only">{{ title }}</legend>
    <h2 class="text-xl font-bold">{{ title }}</h2>
    <p class="mt-2 font-sans text-sm leading-relaxed text-light-subtle dark:text-dark-subtle">
      {{ description }}
    </p>
    <div class="mt-6 space-y-5"><slot /></div>
    <p
      v-if="feedback"
      :role="feedback.kind === 'error' ? 'alert' : 'status'"
      :class="
        feedback.kind === 'error'
          ? 'text-light-love dark:text-dark-love'
          : 'text-light-pine dark:text-dark-foam'
      "
      class="mt-4 font-sans text-sm"
    >
      {{ feedback.message }}
    </p>
    <div class="mt-6 flex flex-wrap gap-3">
      <Button
        type="submit"
        variant="primary"
        size="sm"
        :loading="saving"
        :disabled="disabled || !dirty"
        >{{ saving ? "Saving…" : "Save changes" }}</Button
      >
      <Button type="button" size="sm" :disabled="disabled || !dirty" @click="emit('discard')"
        >Discard changes</Button
      >
    </div>
  </fieldset>
</template>
