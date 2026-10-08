<script setup lang="ts">
const open = ref(false);

onKeyStroke(
  ["c", "C"],
  (event) => {
    const shouldIgnore = [
      event.ctrlKey,
      event.metaKey,
      event.altKey,
      event.isComposing,
      event.defaultPrevented,
    ].some(Boolean);

    if (shouldIgnore || open.value) {
      return;
    }

    const target = event.target;
    const isEditing =
      target instanceof HTMLElement &&
      (target.isContentEditable || target.closest('input, textarea, select, [role="textbox"]'));

    if (isEditing) {
      return;
    }

    if (document.querySelector('[role="dialog"][data-state="open"]')) {
      return;
    }

    event.preventDefault();
    open.value = true;
  },
  { dedupe: true },
);
</script>

<template>
  <TaskCreator v-model:open="open">
    <template #trigger>
      <button
        type="button"
        aria-label="Create task"
        aria-keyshortcuts="C"
        class="fixed right-6 bottom-6 z-30 flex cursor-pointer items-center gap-2 rounded-md border-2 border-transparent bg-light-pine px-4 py-3 text-light-base shadow-lg transition hover:brightness-110 focus:border-light-text focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-light-text dark:bg-dark-pine dark:text-dark-text dark:focus:border-dark-text dark:focus-visible:outline-dark-text"
      >
        <Icon name="carbon:add" class="size-6" />
        <span>Add task</span>
        <ShortcutKey shortcut="C" class="ml-1" />
      </button>
    </template>
  </TaskCreator>
</template>
