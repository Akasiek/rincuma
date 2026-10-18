<script setup lang="ts">
import Control from "./Control.vue";

const model = defineModel<string | null>({ default: "" });
const { type = "text", variant = "default" } = defineProps<{
  type?: string;
  variant?: "default" | "task";
}>();
</script>

<template>
  <Control v-model="model" :type="type" :variant="variant" />
</template>
