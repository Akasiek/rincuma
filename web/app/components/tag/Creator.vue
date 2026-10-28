<script setup lang="ts">
import type { Tag } from "#shared/types/tag";
import type { DictionaryFormValues } from "~/utils/dictionaryCreation";

const open = defineModel<boolean>("open", { default: false });
const emit = defineEmits<{ created: [tag: Tag] }>();
const { createTag } = useDictionaries();
const userStore = useUserStore();

async function create({ name, color }: DictionaryFormValues) {
  const tag = await createTag({ name, color });
  if (userStore.user?.id === tag.owner_id) {
    emit("created", tag);
  }
}
</script>

<template>
  <DictionaryForm v-model:open="open" kind="tag" :save="create">
    <template v-if="$slots.trigger" #trigger><slot name="trigger" /></template>
  </DictionaryForm>
</template>
