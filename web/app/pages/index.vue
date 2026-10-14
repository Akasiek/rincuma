<script setup lang="ts">
defineOptions({ name: "AllTasksPage" });
useHead({ title: "All tasks" });
</script>

<template>
  <TaskWorkspace>
    <TaskList />
  </TaskWorkspace>
</template>
