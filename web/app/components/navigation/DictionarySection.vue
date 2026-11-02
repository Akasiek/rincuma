<script setup lang="ts">
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger, useToastManager } from "reka-ui";
import type { NavigationSection } from "#shared/types/navigationPreferences";

const props = defineProps<{ section: NavigationSection }>();
const emit = defineEmits<{ navigate: [] }>();
const store = useDictionariesStore();
const preferences = useNavigationPreferencesStore();
const route = useRoute();
const toast = useToastManager();
const label = computed(() => (props.section === "projects" ? "Projects" : "Tags"));
const icon = computed(() => (props.section === "projects" ? "carbon:folder" : "carbon:tag"));
const records = computed(() => {
  const values =
    props.section === "projects"
      ? store.projects.filter((project) => !project.archived_at)
      : store.tags;
  return [...values].sort(
    (left, right) => left.name.localeCompare(right.name) || left.id - right.id,
  );
});
const open = computed(() => preferences.isOpen(props.section, records.value.length));
const active = computed(() => route.path.startsWith(`/${props.section}/`));

async function toggle(value: boolean) {
  try {
    await preferences.setOpen(props.section, value);
  } catch {
    toast.add({
      title: "Could not save navigation preferences",
      description: "Please try again.",
      status: "error",
    });
  }
}
</script>

<template>
  <CollapsibleRoot :open="open" @update:open="toggle">
    <CollapsibleTrigger
      :disabled="preferences.saving[section]"
      :class="{ 'bg-light-highlight-med/50 dark:bg-dark-highlight-med/50': active }"
      class="flex w-full cursor-pointer items-center gap-2.5 rounded-md px-3 py-2.5 text-left font-serif text-sm hover:bg-light-highlight-med disabled:cursor-wait dark:hover:bg-dark-highlight-med"
    >
      <Icon :name="icon" class="size-4 shrink-0" aria-hidden="true" />
      <span class="flex-1">{{ label }}</span>
      <Icon
        name="carbon:chevron-down"
        :class="{ '-rotate-90': !open }"
        class="size-4 shrink-0 transition-transform motion-reduce:transition-none"
        aria-hidden="true"
      />
    </CollapsibleTrigger>
    <CollapsibleContent>
      <nav v-if="records.length" :aria-label="label" class="mt-1 space-y-1 pl-3 font-serif text-sm">
        <NuxtLink
          v-for="record in records"
          :key="record.id"
          :to="`/${section}/${record.id}`"
          :title="record.name"
          class="flex min-w-0 items-center gap-2.5 rounded-md px-3 py-2.5 hover:bg-light-highlight-med dark:hover:bg-dark-highlight-med"
          exact-active-class="bg-light-highlight-med/50 dark:bg-dark-highlight-med/50"
          @click="emit('navigate')"
        >
          <span
            :style="{ backgroundColor: record.color || undefined }"
            class="size-2.5 shrink-0 rounded-full bg-light-muted dark:bg-dark-muted"
            aria-hidden="true"
          />
          <span class="truncate">{{ record.name }}</span>
        </NuxtLink>
      </nav>
      <p v-else class="px-6 py-2 font-sans text-xs text-light-subtle dark:text-dark-subtle">
        {{ section === "projects" ? "No projects" : "No tags" }}
      </p>
    </CollapsibleContent>
  </CollapsibleRoot>
</template>
