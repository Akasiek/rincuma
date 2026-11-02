<script setup lang="ts">
const route = useRoute();
definePageMeta({ key: (route) => route.path, taskDictionary: "project" });
</script>

<template>
  <DictionaryTaskView kind="project" :id="String(route.params.id)" />
</template>
