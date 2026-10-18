<script setup lang="ts">
import Control from "./Control.vue";

const model = defineModel<string | null>({ default: "" });
const { variant = "default" } = defineProps<{ variant?: "default" | "task" }>();
</script>

<template>
  <Control v-model="model" as="textarea" :variant="variant" class="min-h-26 resize-y" />
</template>
