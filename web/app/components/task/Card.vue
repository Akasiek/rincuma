<script setup lang="ts">
import type { Task } from "#shared/types/task";
import Summary from "./card/Summary.vue";
import Metadata from "./card/Metadata.vue";
import Description from "./card/Description.vue";
import CompletionButton from "./card/CompletionButton.vue";

const props = defineProps<{ task: Task }>();
const emit = defineEmits<{ updated: [task: Task] }>();
const { completeTask, reopenTask } = useTask();
const { showCompletedTask, dismissCompletedTask } = useTaskCompletionToast();
const isUpdatingCompletion = ref(false);
const completionError = ref("");
const descriptionOpen = ref(false);
const descriptionId = useId();

async function toggleCompletion() {
  if (isUpdatingCompletion.value) {
    return;
  }

  isUpdatingCompletion.value = true;
  completionError.value = "";

  try {
    const updateCompletion = props.task.completed_at ? reopenTask : completeTask;
    const task = await updateCompletion(props.task.id);
    if (task.completed_at) {
      showCompletedTask(task);
    } else {
      dismissCompletedTask(task);
    }
    emit("updated", task);
  } catch (error) {
    const status = (error as { response?: { status?: number } })?.response?.status;
    completionError.value =
      status === 409 && props.task.completed_at
        ? "Cannot reopen this occurrence because its series or next occurrence has changed."
        : "Could not change the task status. Please try again.";
  } finally {
    isUpdatingCompletion.value = false;
  }
}
</script>

<template>
  <article
    class="group relative rounded-md px-2 py-2.5 transition-colors focus-within:bg-light-highlight-low/70 hover:bg-light-highlight-low/70 dark:focus-within:bg-dark-highlight-low dark:hover:bg-dark-highlight-low"
  >
    <div
      class="pointer-events-none absolute right-2 bottom-0 left-10 border-b border-light-highlight-med/60 dark:border-dark-highlight-med/60"
    />
    <div class="flex items-start gap-2">
      <CompletionButton
        class="-mt-0.5 shrink-0"
        :completed="!!task.completed_at"
        :saving="isUpdatingCompletion"
        :priority="task.priority"
        @toggle="toggleCompletion"
      />
      <div class="min-w-0 flex-1">
        <div class="flex flex-col gap-x-5 gap-y-1.5 sm:flex-row sm:items-start">
          <Summary
            v-model:description-open="descriptionOpen"
            :task="task"
            :description-id="descriptionId"
          />
          <Metadata :task="task" />
        </div>
        <Description
          v-if="task.description"
          :id="descriptionId"
          :description="task.description"
          :open="descriptionOpen"
        />
        <p
          v-if="completionError"
          role="alert"
          class="mt-2 text-xs text-light-love dark:text-dark-love"
        >
          {{ completionError }}
        </p>
      </div>
    </div>
  </article>
</template>
