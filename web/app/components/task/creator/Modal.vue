<script setup lang="ts">
import ErrorCard from "./ErrorCard.vue";

const open = defineModel<boolean>("open", { default: false });
const {
  formId,
  errors = [],
  isSubmitting = false,
} = defineProps<{
  formId: string;
  errorSummaryId: string;
  errors?: { fieldId?: string; message: string }[];
  isSubmitting?: boolean;
}>();

function focusField(fieldId: string) {
  document.getElementById(fieldId)?.focus();
}

function focusName(event: Event) {
  event.preventDefault();
  focusField(`${formId}-name`);
}
</script>

<template>
  <BaseModal
    v-model:open="open"
    title="New Task"
    @open-auto-focus="focusName"
    class="task-creator-modal isolate flex max-h-[calc(100dvh-6rem)] w-[calc(100%-2rem)] max-w-lg flex-col overflow-visible font-serif text-light-text dark:text-dark-text"
  >
    <template v-if="$slots.trigger" #trigger>
      <slot name="trigger" />
    </template>
    <div aria-hidden="true" class="pointer-events-none absolute inset-0 z-0">
      <div
        class="h-full w-full -translate-x-1 rotate-5 rounded-md bg-light-base dark:bg-dark-base"
      />
    </div>
    <DecorationsPaperclipBack
      aria-hidden="true"
      class="pointer-events-none absolute -top-8 left-1/3 -z-10 h-auto w-8 rotate-24 drop-shadow-[1px_2px_1px_#00000033]"
    />
    <div
      class="relative z-10 min-h-0 overflow-y-auto rounded-md border-2 border-transparent bg-light-base shadow-[0_0_1em_#00000088] dark:bg-dark-base"
    >
      <slot />
    </div>

    <ErrorCard :summary-id="errorSummaryId" :errors="errors" @focus-field="focusField" />

    <DecorationsPaperclipFront
      aria-hidden="true"
      class="pointer-events-none absolute -top-8 left-1/3 z-15 h-auto w-8 rotate-24 drop-shadow-[1px_2px_1px_#00000033]"
    />

    <button
      type="button"
      aria-label="Close task creator"
      class="absolute -top-4 -right-4 z-20 flex size-10 -rotate-2 cursor-pointer items-center justify-center rounded-md border-2 border-transparent bg-light-love shadow-[0_0_1em_#00000044] brightness-90 transition hover:scale-105 hover:rotate-0 hover:brightness-100 focus:border-light-text focus:outline-none focus-visible:scale-105 focus-visible:rotate-0 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-light-text focus-visible:brightness-100 dark:bg-dark-love dark:focus:border-dark-text dark:focus-visible:outline-dark-text"
      @click="() => (open = false)"
    >
      <Icon name="carbon:close" class="size-8 text-light-base dark:text-dark-base" />
    </button>

    <button
      type="submit"
      :form="formId"
      :disabled="isSubmitting"
      :aria-busy="isSubmitting"
      aria-label="Save task"
      class="absolute -right-6 -bottom-4 z-20 flex size-12 rotate-1 cursor-pointer items-center justify-center rounded-md border-2 border-transparent bg-light-base shadow-[0_0_1em_#00000044] brightness-90 transition hover:scale-105 hover:rotate-0 hover:brightness-100 focus:border-light-text focus:outline-none focus-visible:scale-105 focus-visible:rotate-0 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-light-text focus-visible:brightness-100 disabled:cursor-wait disabled:opacity-60 dark:bg-dark-pine dark:focus:border-dark-text dark:focus-visible:outline-dark-text"
    >
      <Icon name="carbon:save" class="size-8 text-light-text dark:text-dark-text" />
    </button>
  </BaseModal>
</template>

<style>
.task-creator-modal[data-state="open"] {
  animation: task-creator-in 220ms ease-out;
}
.task-creator-modal[data-state="closed"] {
  animation: task-creator-out 140ms ease-in forwards;
}
@keyframes task-creator-in {
  from {
    opacity: 0;
    transform: translateY(12px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
@keyframes task-creator-out {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(12px) scale(0.98);
  }
}
@media (prefers-reduced-motion: reduce) {
  .task-creator-modal {
    animation: none !important;
  }
}
</style>
