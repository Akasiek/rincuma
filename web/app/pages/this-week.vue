<script setup lang="ts">
defineOptions({ name: "NextSevenDaysPage" });
</script>

<template>
  <TaskWorkspace>
    <TaskList
      title="Next 7 days"
      :filters="{ period: 'next-seven-days', sort_by: 'due_at', order: 'asc' }"
    />
  </TaskWorkspace>
</template>
