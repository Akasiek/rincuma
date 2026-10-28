<script setup lang="ts">
import type { Project } from "#shared/types/project";
import type { DictionaryFormValues } from "~/utils/dictionaryCreation";

const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ created: [project: Project] }>();
const { createProject } = useDictionaries();
const userStore = useUserStore();

async function create(values: DictionaryFormValues) {
  const project = await createProject({
    ...values,
    description: values.description.trim() ? values.description : null,
  });

  if (userStore.user?.id === project.owner_id) {
    emit("created", project);
  }
}
</script>

<template>
  <DictionaryForm v-model:open="open" kind="project" :save="create">
    <template v-if="$slots.trigger" #trigger><slot name="trigger" /></template>
  </DictionaryForm>
</template>
