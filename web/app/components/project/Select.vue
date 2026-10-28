<script setup lang="ts">
import type { SelectModelValue } from "#shared/types/select.ts";
import { storeToRefs } from "pinia";

const props = defineProps<{
  id: string;
  retainedProjectId?: number | null;
  compact?: boolean;
  disabled?: boolean;
  invalid?: boolean;
  describedby?: string;
}>();

const creating = ref(false);

const model = defineModel<number | null>({ default: null });
const { projects, loaded } = storeToRefs(useDictionariesStore());

const options = computed(() => [
  { value: null, label: "No project" },
  ...projects.value
    .filter((project) => !project.archived_at || project.id === props.retainedProjectId)
    .map((project) => ({
      value: project.id,
      label: project.archived_at ? `${project.name} (archived)` : project.name,
    })),
]);

const selectedProject = computed(() =>
  projects.value.find((project) => project.id === model.value),
);

const triggerClass = computed(() =>
  model.value === null
    ? "text-light-subtle dark:text-dark-subtle"
    : "text-light-iris dark:text-dark-iris",
);

const triggerStyle = computed(() =>
  !props.compact || model.value === null
    ? undefined
    : {
        color: selectedProject.value?.color || undefined,
        "--select-trigger-background": "color-mix(in srgb, currentColor 10%, transparent)",
        "--select-trigger-hover-background": "color-mix(in srgb, currentColor 20%, transparent)",
      },
);

function updateProject(value: SelectModelValue) {
  if (value === null || typeof value === "number") {
    model.value = value;
  }
}

function projectColor(value: SelectModelValue) {
  return projects.value.find((project) => project.id === value)?.color ?? undefined;
}
function focusTrigger(event: Event) {
  event.preventDefault();
  document.getElementById(props.id)?.focus();
}
</script>

<template>
  <div class="min-w-0">
    <AppSelect
      :id="id"
      :compact="compact"
      :invalid="invalid"
      :describedby="describedby"
      :trigger-class="compact ? triggerClass : undefined"
      :trigger-style="triggerStyle"
      label="Project"
      create-label="Create project…"
      @create="creating = true"
      :model-value="model"
      :options="options"
      placeholder="No project"
      :disabled="disabled || !loaded"
      @update:model-value="updateProject"
    >
      <template #trigger>
        <span class="relative flex size-5 shrink-0 items-center justify-center" aria-hidden="true">
          <Icon name="carbon:folder" class="relative size-5" />
        </span>
        <span
          v-if="selectedProject"
          class="relative min-w-0 truncate text-light-text dark:text-dark-text"
        >
          {{ selectedProject.name }}
        </span>
      </template>
      <template #option-icon="{ option }">
        <span
          v-if="projectColor(option.value)"
          :style="{ backgroundColor: projectColor(option.value) }"
          class="mr-2 size-2.5 shrink-0 rounded-full ring-1 ring-black/10 dark:ring-white/20"
          aria-hidden="true"
        />
      </template>
    </AppSelect>
    <ProjectCreator
      v-model:open="creating"
      @created="model = $event.id"
      @close-auto-focus="focusTrigger"
    />
  </div>
</template>
