<script setup lang="ts">
defineProps<{ shortcut: string }>();
</script>

<template>
  <kbd
    aria-hidden="true"
    class="inline-flex h-6 min-w-6 items-center justify-center rounded border border-b-2 border-current/25 bg-white/10 px-1.5 font-mono text-xs font-semibold shadow-[0_1px_2px_#0000001a]"
  >
    {{ shortcut }}
  </kbd>
</template>
