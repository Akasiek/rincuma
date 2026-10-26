<script setup lang="ts">
defineProps<{ saving: boolean }>();

const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ decision: [leave: boolean] }>();
</script>

<template>
  <AppModal
    v-model:open="open"
    title="Unsaved changes"
    description="You have unsaved changes. Stay or discard them and leave."
    size="sm"
    layer="confirmation"
  >
    <div class="p-6">
      <h2 class="text-xl font-bold">Unsaved changes</h2>
      <p class="mt-3 font-sans text-sm leading-relaxed text-light-subtle dark:text-dark-subtle">
        {{
          saving
            ? "Your changes are being saved. Please wait before leaving."
            : "You have unsaved changes. Discard them and leave?"
        }}
      </p>
      <div class="mt-6 flex flex-wrap gap-3">
        <Button @click="emit('decision', false)">Stay</Button>
        <Button variant="danger" :disabled="saving" @click="emit('decision', true)"
          >Discard and leave</Button
        >
      </div>
    </div>
  </AppModal>
</template>
