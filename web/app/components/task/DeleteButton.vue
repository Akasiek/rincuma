<script setup lang="ts">
import type { Task } from "#shared/types/task";

const props = defineProps<{ task: Task; disabled?: boolean; dirty?: boolean }>();
const emit = defineEmits<{ deleted: []; deleting: [deleting: boolean] }>();
const open = ref(false);
const { deleting, error, remove } = useTaskDeletion(toRef(props, "task"), () => {
  open.value = false;
  emit("deleted");
});
const stopsSeries = computed(() => !!props.task.recurrence && !props.task.completed_at);

watch(deleting, (value) => emit("deleting", value), { flush: "sync" });

function updateOpen(value: boolean) {
  if (!deleting.value) {
    open.value = value;
    error.value = "";
  }
}

function onEscape(event: KeyboardEvent) {
  if (deleting.value) {
    event.preventDefault();
  }
}
</script>

<template>
  <AppModal
    :open="open"
    title="Delete task?"
    description="Delete this task and any subtasks?"
    size="sm"
    :aria-busy="deleting"
    @update:open="updateOpen"
    @escape-key-down="onEscape"
    @interact-outside.prevent
  >
    <template #trigger>
      <button
        type="button"
        :disabled="disabled || deleting"
        aria-label="Delete task"
        title="Delete task"
        class="flex size-10 rotate-2 cursor-pointer items-center justify-center rounded-md border-2 border-transparent bg-light-base text-light-love shadow-[0_0_1em_#00000044] transition hover:scale-105 hover:rotate-0 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-dark-base dark:text-dark-love"
      >
        <Icon name="carbon:trash-can" class="size-5" aria-hidden="true" />
      </button>
    </template>
    <div class="p-6">
      <h2 class="text-xl font-bold">Delete task?</h2>
      <div
        class="mt-3 space-y-3 font-sans text-sm leading-relaxed text-light-subtle dark:text-dark-subtle"
      >
        <p>Delete “{{ task.name }}” and any subtasks?</p>
        <p v-if="stopsSeries">
          This will stop the repeating series. Previous completed occurrences will be kept.
        </p>
        <p v-else-if="!task.series_id">
          Repeating series for any deleted active subtasks will also stop.
        </p>
        <p v-if="dirty">Unsaved edits will be discarded.</p>
      </div>
      <p
        v-if="error"
        role="alert"
        class="mt-3 font-sans text-sm text-light-love dark:text-dark-love"
      >
        {{ error }}
      </p>
      <div class="mt-6 flex justify-end gap-3">
        <Button :disabled="deleting" @click="updateOpen(false)">Cancel</Button>
        <Button variant="danger" :loading="deleting" @click="remove">Delete task</Button>
      </div>
    </div>
  </AppModal>
</template>
