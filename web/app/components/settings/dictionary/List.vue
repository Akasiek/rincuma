<script setup lang="ts">
defineProps<{ kind: "project" | "tag"; records: (Project | Tag)[] }>();
const emit = defineEmits<{ edit: [record: Project | Tag]; delete: [record: Project | Tag] }>();

function recordDescription(record: Project | Tag) {
  return "description" in record ? record.description : null;
}

function isArchived(record: Project | Tag) {
  return "archived_at" in record && !!record.archived_at;
}
</script>

<template>
  <div
    class="overflow-hidden rounded-lg border border-light-highlight-med bg-light-surface dark:border-dark-highlight-med dark:bg-dark-surface"
  >
    <ul class="divide-y divide-light-highlight-med dark:divide-dark-highlight-med">
      <li
        v-for="record in records"
        :key="record.id"
        class="flex items-center gap-3 px-4 py-3 transition-colors hover:bg-light-base/60 sm:gap-4 dark:hover:bg-dark-base/60"
      >
        <span
          :style="
            record.color
              ? {
                  color: record.color,
                  backgroundColor: 'color-mix(in srgb, currentColor 12%, transparent)',
                }
              : undefined
          "
          class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-light-highlight-low text-light-subtle dark:bg-dark-highlight-low dark:text-dark-subtle"
          aria-hidden="true"
        >
          <Icon :name="kind === 'project' ? 'carbon:folder' : 'carbon:tag'" class="size-5" />
        </span>

        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
            <button
              type="button"
              :aria-label="`Edit ${record.name}`"
              class="min-w-0 cursor-pointer text-left font-semibold break-words decoration-light-subtle/50 underline-offset-4 hover:underline dark:decoration-dark-subtle/50"
              @click="emit('edit', record)"
            >
              {{ record.name }}
            </button>
            <span
              class="inline-flex h-6 shrink-0 items-center gap-1.5 rounded-md bg-light-base px-2 font-sans text-xs leading-none text-light-subtle tabular-nums dark:bg-dark-base dark:text-dark-subtle"
              :aria-label="`${record.task_count ?? 0} ${record.task_count === 1 ? 'task' : 'tasks'}`"
              title="Each repeating series counts once. Includes completed tasks; excludes deleted tasks."
            >
              <Icon name="carbon:task" class="block size-3.5 shrink-0" aria-hidden="true" />
              <span class="mt-0.5 inline-flex h-3.5 items-center leading-none">
                {{ record.task_count ?? 0 }}
              </span>
            </span>
            <span
              v-if="isArchived(record)"
              class="rounded-sm bg-light-highlight-low px-1.5 py-0.5 font-sans text-[10px] text-light-subtle dark:bg-dark-highlight-low dark:text-dark-subtle"
            >
              Archived
            </span>
          </div>
          <p
            v-if="recordDescription(record)"
            class="mt-0.5 line-clamp-1 font-sans text-xs leading-relaxed break-words text-light-subtle dark:text-dark-subtle"
          >
            {{ recordDescription(record) }}
          </p>
        </div>

        <SettingsDictionaryActions
          :name="record.name"
          @edit="emit('edit', record)"
          @delete="emit('delete', record)"
        />
      </li>
    </ul>
  </div>
</template>
