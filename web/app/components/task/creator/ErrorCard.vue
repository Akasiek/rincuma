<script setup lang="ts">
defineProps<{
  summaryId: string;
  errors: { fieldId?: string; message: string }[];
}>();

const emit = defineEmits<{ focusField: [fieldId: string] }>();
</script>

<template>
  <Transition
    enter-active-class="transition duration-200 ease-out motion-reduce:transition-none"
    enter-from-class="-translate-y-full opacity-0"
    enter-to-class="translate-y-0 opacity-100"
    leave-active-class="transition duration-150 ease-in motion-reduce:transition-none"
    leave-from-class="translate-y-0 opacity-100"
    leave-to-class="-translate-y-full opacity-0"
  >
    <div
      v-if="errors.length"
      :id="summaryId"
      role="alert"
      class="relative z-5 -mt-2 mr-12 ml-6 max-h-[30dvh] shrink-0 overflow-y-auto rounded-md bg-light-love px-4 pt-5 pb-4 font-sans text-sm text-light-surface shadow-md dark:bg-dark-love dark:text-dark-base"
    >
      <p class="mb-2 font-bold">Please check the task</p>
      <ul class="list-inside list-disc space-y-1">
        <li v-for="error in errors" :key="`${error.fieldId ?? 'save'}-${error.message}`">
          <button
            v-if="error.fieldId"
            type="button"
            class="cursor-pointer text-left underline decoration-dotted underline-offset-4 focus-visible:outline-2 focus-visible:outline-current"
            @click="emit('focusField', error.fieldId)"
          >
            {{ error.message }}
          </button>
          <span v-else>{{ error.message }}</span>
        </li>
      </ul>
    </div>
  </Transition>
</template>
