<script setup lang="ts">
const route = useRoute();
definePageMeta({ key: (route) => route.path, taskDictionary: "tag" });
</script>

<template>
  <DictionaryTaskView kind="tag" :id="String(route.params.id)" />
</template>
