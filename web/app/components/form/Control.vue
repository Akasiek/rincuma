<script setup lang="ts">
import { FieldControl, FieldRoot } from "reka-ui";

defineOptions({ inheritAttrs: false });

const model = defineModel<string | null>({ default: "" });
const {
  as = "input",
  disabled = false,
  invalid = false,
  variant = "default",
} = defineProps<{
  as?: "input" | "textarea";
  disabled?: boolean;
  invalid?: boolean;
  variant?: "default" | "task";
}>();
const controlClass = computed(() => {
  if (variant === "task") {
    const sizeClass = as === "textarea" ? "px-4 py-2 text-lg" : "p-4 text-2xl font-semibold";
    return [
      "border-0 bg-transparent font-serif aria-invalid:ring-2 aria-invalid:ring-light-love dark:aria-invalid:ring-dark-love",
      sizeClass,
    ];
  }
  return "border-2 border-light-highlight-med bg-light-base px-3 py-2.5 font-sans text-sm aria-invalid:border-light-love dark:border-dark-highlight-med dark:bg-dark-base dark:aria-invalid:border-dark-love";
});

const value = computed({
  get: () => model.value ?? "",
  set: (next: string) => {
    model.value = next;
  },
});
</script>

<template>
  <FieldRoot as-child :disabled="disabled" :invalid="invalid">
    <FieldControl
      v-bind="$attrs"
      v-model="value"
      :as="as"
      :aria-invalid="invalid"
      :class="controlClass"
      class="w-full rounded-md disabled:cursor-not-allowed disabled:opacity-50"
    />
  </FieldRoot>
</template>
