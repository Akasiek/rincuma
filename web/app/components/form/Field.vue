<script setup lang="ts">
import { Field } from "vee-validate";
const { as = "input" } = defineProps<{
  name: string;
  label: string;
  as?: string;
}>();
</script>

<template>
  <div class="flex flex-col gap-1">
    <label for="name">{{ label }}</label>
    <Field
      :id="name"
      :name="name"
      :as="as"
      class="border-2 border-light-highlight-med dark:border-dark-highlight-med rounded-md px-3 py-1.5 focus:dark:outline-0 focus:dark:border-dark-highlight-high"
      :label="label"
    />
    <ErrorMessage :name="name" />
  </div>
</template>
