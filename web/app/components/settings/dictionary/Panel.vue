<script setup lang="ts">
import { useToastManager } from "reka-ui";
import type { DictionaryFormValues } from "~/utils/dictionaryCreation";

const props = defineProps<{ kind: "project" | "tag" }>();
const store = useDictionariesStore();
const userStore = useUserStore();
const { loadDictionaries, updateProject, updateTag, deleteRecord } = useDictionaries();
const { refreshTaskLists } = useTask();
const toast = useToastManager();
const key = computed(() => `dictionary-settings:${props.kind}:${userStore.user?.id ?? "guest"}`);
const { error: loadError, refresh } = await useAsyncData(key, async () => {
  await loadDictionaries(true);
  return true;
});
const title = computed(() => (props.kind === "project" ? "Projects" : "Tags"));
const records = computed(() => (props.kind === "project" ? store.projects : store.tags));
const editing = ref<Project | Tag | null>(null);
const editOpen = ref(false);
const deleting = ref<Project | Tag | null>(null);
const removing = ref(false);
const deleteError = ref("");
const initialValues = computed<DictionaryFormValues | undefined>(() => {
  const record = editing.value;
  if (!record) {
    return undefined;
  }
  return {
    name: record.name,
    description: "description" in record ? (record.description ?? "") : "",
    color: record.color,
  };
});

useUnsavedChanges({ dirty: computed(() => false), saving: removing });

function edit(record: Project | Tag) {
  editing.value = record;
  editOpen.value = true;
}

async function refreshTasks() {
  try {
    await refreshTaskLists();
  } catch {
    toast.add({
      title: "Changes saved",
      description: "Reload to update your task lists.",
      status: "error",
    });
  }
}

async function save(values: DictionaryFormValues) {
  const record = editing.value;
  if (!record) {
    return;
  }
  if (props.kind === "project") {
    await updateProject(record.id, { ...values, description: values.description.trim() || null });
  } else {
    await updateTag(record.id, { name: values.name, color: values.color });
  }
  await refreshTasks();
}

function confirmDelete(record: Project | Tag) {
  deleteError.value = "";
  deleting.value = record;
}

function closeDelete(open: boolean) {
  if (!open && !removing.value) {
    deleting.value = null;
  }
}

function preventClose(event: Event) {
  if (removing.value) {
    event.preventDefault();
  }
}

async function remove() {
  const record = deleting.value;
  if (!record || removing.value) {
    return;
  }
  removing.value = true;
  deleteError.value = "";
  try {
    await deleteRecord(props.kind, record.id);
  } catch {
    deleteError.value = `Could not delete the ${props.kind}. Please try again.`;
    removing.value = false;
    return;
  }
  deleting.value = null;
  removing.value = false;
  toast.add({
    title: props.kind === "project" ? "Project deleted" : "Tag deleted",
    description: record.name,
    status: "success",
  });
  await refreshTasks();
}
</script>

<template>
  <section :aria-label="title">
    <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2.5">
          <h2 class="text-xl font-bold">{{ title }}</h2>
          <span
            v-if="!loadError"
            class="rounded-md bg-light-highlight-low px-2 py-0.5 font-sans text-xs text-light-subtle tabular-nums dark:bg-dark-highlight-low dark:text-dark-subtle"
          >
            {{ records.length }}
          </span>
        </div>
        <p class="mt-2 font-sans text-sm text-light-subtle dark:text-dark-subtle">
          {{
            kind === "project"
              ? "Organize related tasks into projects."
              : "Keep your tasks organized with labels."
          }}
        </p>
      </div>
      <ProjectCreator v-if="kind === 'project'">
        <template #trigger>
          <Button variant="primary">
            <Icon name="carbon:add" class="size-4" aria-hidden="true" />
            New project
          </Button>
        </template>
      </ProjectCreator>
      <TagCreator v-else>
        <template #trigger>
          <Button variant="primary">
            <Icon name="carbon:add" class="size-4" aria-hidden="true" />
            New tag
          </Button>
        </template>
      </TagCreator>
    </div>

    <div v-if="loadError" class="space-y-3">
      <p role="alert" class="font-sans text-sm text-light-love dark:text-dark-love">
        Could not load {{ title.toLowerCase() }}.
      </p>
      <Button @click="refresh()">Try again</Button>
    </div>

    <div
      v-else-if="!records.length"
      class="flex flex-col items-center rounded-lg border border-dashed border-light-highlight-med px-6 py-12 text-center dark:border-dark-highlight-med"
    >
      <span
        class="mb-4 flex size-11 items-center justify-center rounded-xl bg-light-highlight-low text-light-subtle dark:bg-dark-highlight-low dark:text-dark-subtle"
        aria-hidden="true"
      >
        <Icon :name="kind === 'project' ? 'carbon:folder' : 'carbon:tag'" class="size-6" />
      </span>
      <h3 class="font-semibold">No {{ title.toLowerCase() }} yet</h3>
      <p class="mt-1 font-sans text-sm text-light-subtle dark:text-dark-subtle">
        Create your first {{ kind }} to get started.
      </p>
    </div>

    <SettingsDictionaryList
      v-else
      :kind="kind"
      :records="records"
      @edit="edit"
      @delete="confirmDelete"
    />
    <DictionaryForm
      v-model:open="editOpen"
      :kind="kind"
      :initial-values="initialValues"
      :save="save"
    />
    <AppModal
      :open="!!deleting"
      :title="`Delete ${kind}?`"
      size="sm"
      layer="confirmation"
      :aria-busy="removing"
      @update:open="closeDelete"
      @escape-key-down="preventClose"
      @interact-outside="preventClose"
    >
      <div class="p-6">
        <h2 class="text-xl font-bold">Delete {{ kind }}?</h2>
        <p class="mt-3 font-sans text-sm break-words">
          Delete “{{ deleting?.name }}”? Tasks will be kept; their {{ kind }} assignment will be
          removed. This cannot be undone.
        </p>
        <p
          v-if="deleteError"
          role="alert"
          class="mt-3 font-sans text-sm text-light-love dark:text-dark-love"
        >
          {{ deleteError }}
        </p>
        <div class="mt-6 flex justify-end gap-3">
          <Button :disabled="removing" @click="closeDelete(false)">Cancel</Button>
          <Button variant="danger" :loading="removing" @click="remove">Delete {{ kind }}</Button>
        </div>
      </div>
    </AppModal>
  </section>
</template>
